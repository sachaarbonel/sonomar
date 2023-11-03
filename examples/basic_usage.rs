use anyhow::{Error as E, Result};
use candle_core::Tensor;
use candle_nn::VarBuilder;

use clap::Parser;
use hf_hub::{api::sync::Api, Repo, RepoType};
use sonomar::{device, multilingual, token_id, Args, Decoder, Model, WhichModel};
use tokenizers::Tokenizer;

use candle_transformers::models::whisper::{self as m, audio, Config};

fn main() -> Result<()> {
    let args = Args::parse();
    // let _guard = if args.tracing {
    //     let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
    //     tracing_subscriber::registry().with(chrome_layer).init();
    //     Some(guard)
    // } else {
    //     None
    // };
    let device = device(args.cpu)?;
    let (default_model, default_revision) = if args.quantized {
        ("lmz/candle-whisper", "main")
    } else {
        args.model.model_and_revision()
    };
    let default_model = default_model.to_string();
    let default_revision = default_revision.to_string();
    let (model_id, revision) = match (args.model_id, args.revision) {
        (Some(model_id), Some(revision)) => (model_id, revision),
        (Some(model_id), None) => (model_id, "main".to_string()),
        (None, Some(revision)) => (default_model, revision),
        (None, None) => (default_model, default_revision),
    };

    let (config_filename, tokenizer_filename, weights_filename, input) = {
        let api = Api::new()?;
        let dataset = api.dataset("Narsil/candle-examples".to_string());
        let repo = api.repo(Repo::with_revision(model_id, RepoType::Model, revision));
        let sample = if let Some(input) = args.input {
            if let Some(sample) = input.strip_prefix("sample:") {
                dataset.get(&format!("samples_{sample}.wav"))?
            } else {
                std::path::PathBuf::from(input)
            }
        } else {
            println!("No audio file submitted: Downloading https://huggingface.co/datasets/Narsil/candle_demo/blob/main/samples_jfk.wav");
            dataset.get("samples_jfk.wav")?
        };
        let (config, tokenizer, model) = if args.quantized {
            let ext = match args.model {
                WhichModel::TinyEn => "tiny-en",
                WhichModel::Tiny => "tiny",
                _ => unimplemented!("no quantized support for {:?}", args.model),
            };
            (
                repo.get(&format!("config-{ext}.json"))?,
                repo.get(&format!("tokenizer-{ext}.json"))?,
                repo.get(&format!("model-{ext}-q80.gguf"))?,
            )
        } else {
            (
                repo.get("config.json")?,
                repo.get("tokenizer.json")?,
                repo.get("model.safetensors")?,
            )
        };
        (config, tokenizer, model, sample)
    };
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

    let mel_bytes = include_bytes!("melfilters.bytes");
    let mut mel_filters = vec![0f32; mel_bytes.len() / 4];
    <byteorder::LittleEndian as byteorder::ByteOrder>::read_f32_into(mel_bytes, &mut mel_filters);

    let mut input = std::fs::File::open(input)?;
    let (header, data) = wav::read(&mut input)?;
    println!("loaded wav data: {header:?}");
    if header.sampling_rate != m::SAMPLE_RATE as u32 {
        anyhow::bail!("wav file must have a {} sampling rate", m::SAMPLE_RATE)
    }
    let data = data.as_sixteen().expect("expected 16 bit wav file");
    let pcm_data: Vec<_> = data[..data.len() / header.channel_count as usize]
        .iter()
        .map(|v| *v as f32 / 32768.)
        .collect();
    println!("pcm data loaded {}", pcm_data.len());
    let mel = audio::pcm_to_mel(&pcm_data, &mel_filters);
    let mel_len = mel.len();
    let mel = Tensor::from_vec(mel, (1, m::N_MELS, mel_len / m::N_MELS), &device)?;
    println!("loaded mel: {:?}", mel.dims());

    let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
    let mut model = if args.quantized {
        let vb =
            candle_transformers::quantized_var_builder::VarBuilder::from_gguf(&weights_filename)?;
        Model::Quantized(m::quantized_model::Whisper::load(&vb, config)?)
    } else {
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], m::DTYPE, &device)? };
        Model::Normal(m::model::Whisper::load(&vb, config)?)
    };

    let language_token = match (args.model.is_multilingual(), args.language) {
        (true, None) => Some(multilingual::detect_language(&mut model, &tokenizer, &mel)?),
        (false, None) => None,
        (true, Some(language)) => match token_id(&tokenizer, &format!("<|{language}|>")) {
            Ok(token_id) => Some(token_id),
            Err(_) => anyhow::bail!("language {language} is not supported"),
        },
        (false, Some(_)) => {
            anyhow::bail!("a language cannot be set for non-multilingual models")
        }
    };
    let mut dc = Decoder::new(
        model,
        tokenizer,
        args.seed,
        &device,
        language_token,
        args.task,
        args.timestamps,
        args.verbose,
    )?;
    dc.run(&mel)?;
    Ok(())
}
