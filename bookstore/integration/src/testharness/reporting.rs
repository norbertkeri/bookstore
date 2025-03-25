use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_reporting(frames_to_keep: Vec<String>) {
    color_eyre::config::HookBuilder::default()
        .add_frame_filter(Box::new(move |frames| {
            let f = frames_to_keep.clone();
            frames.retain(move |frame| {
                f.clone().into_iter().any(|f| {
                    let name = if let Some(name) = frame.name.as_ref() {
                        name.as_str()
                    } else {
                        return true;
                    };

                    name.starts_with(&f)
                })
            });
        }))
        .install()
        .expect("Could not install eyre hook");
}

pub fn setup_logging(envfilter: EnvFilter) {
    tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(envfilter)
        .init();
}

pub fn specific_envfilter(default_directives: &str) -> EnvFilter {
    if let Ok(wanted_directive) = std::env::var("RUST_LOG") {
        match EnvFilter::try_new(&wanted_directive) {
            Ok(ok) => ok,
            Err(e) => {
                panic!(
                    "The filtering directives in RUST_LOG ({wanted_directive}) are invalid, could not set up logging: {e}"
                )
            }
        }
    } else {
        EnvFilter::try_new(default_directives).unwrap()
    }
}
