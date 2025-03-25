pub mod dbtools;
pub mod reporting;
pub mod testselector;
use std::panic::AssertUnwindSafe;

use color_eyre::eyre::bail;
use dbtools::create_administrative_database;
use futures::{
    FutureExt, SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender},
    future::{BoxFuture, join_all},
};
use reporting::{setup_logging, setup_reporting, specific_envfilter};
use sqlx::{Pool, Postgres};
use testselector::{TestSelector, create_test_selector};
use tracing::{Instrument, debug, error, info, info_span};
use tracing_subscriber::EnvFilter;

pub struct IntegrationTestCase {
    pub name: &'static str,
    pub fun: fn(TestHarness) -> TestReturn,
}

pub type TestReturn = BoxFuture<'static, color_eyre::Result<()>>;

pub fn run_tests(
    calling_module_name: &'static str,
    default_directives: &str,
) -> color_eyre::Result<()> {
    setup_reporting(vec![
        calling_module_name.to_string(),
        format!("<{calling_module_name}"),
    ]);

    let tests: Box<dyn Iterator<Item = &'static IntegrationTestCase>> =
        match create_test_selector()? {
            TestSelector::Specific(test) => {
                setup_logging(specific_envfilter(default_directives));
                Box::new(std::iter::once(test)) as _
            }
            TestSelector::All(tests) => {
                setup_logging(EnvFilter::from_default_env());
                tests
            }
        };
    let runtime = tokio::runtime::Runtime::new().expect("Can't create a tokio runtime");
    let r = runtime.block_on(async {
        let admin_conn = create_administrative_database().await;
        let mut handles = vec![];
        for case in tests {
            let span = info_span!("test_case", name = &case.name);
            let _guard = span.clone().entered();
            let connection = admin_conn.create_application_pool(case.name).await.unwrap();
            let (mut harness, mut recv) = make_testharness(connection);
            tokio::spawn(
                async {
                    recv.listen().await;
                    recv.clean();
                }
                .instrument(span.clone()),
            );
            info!("Running test");

            let span_clone = span.clone();
            let test_handle = tokio::spawn(
                async move {
                    let result = AssertUnwindSafe((case.fun)(harness.clone()))
                        .catch_unwind()
                        .instrument(span_clone.clone())
                        .await;

                    match result {
                        // test success
                        Ok(Ok(_)) => {
                            debug!("Test success");
                            (case.name, true)
                        }
                        // test failed
                        Ok(Err(e)) => {
                            error!("Test {} has failed: {e:?}", &case.name);
                            harness.mark_failed().await;
                            (case.name, false)
                        }
                        // Thread paniced (most often from a failed assert_eq!() call)
                        // The error here has nothing useful, because it's just a Box<dyn Any>, so
                        // no reason to print it. The error is going to be visible anyway, because
                        // rust prints the panic reason
                        Err(_e) => {
                            harness.mark_failed().await;
                            (case.name, false)
                        }
                    }
                }
                .instrument(span.clone()),
            );
            handles.push(test_handle);
        }

        join_all(handles)
            .await
            .into_iter()
            .all(|handle| match handle {
                Ok((name, is_success)) => {
                    let f = if is_success { "ok" } else { "fail" };
                    println!("test {} ... {}", name, f);
                    is_success
                }
                Err(e) => {
                    // The tasks are running with .catch_unwind(), which means that we should never
                    // encounter a failure when joining on them
                    error!("A test task handle panicked, this should not be possible: {e:?}");
                    false
                }
            })
    });
    if r {
        Ok(())
    } else {
        bail!("Some test(s) have failed")
    }
}

#[derive(Debug, Clone)]
pub struct TestHarness {
    tx: Sender<TestMessage>,
    pub connection: Pool<Postgres>,
}

impl TestHarness {
    pub fn new(tx: Sender<TestMessage>, connection: Pool<Postgres>) -> Self {
        Self { tx, connection }
    }

    pub async fn mark_failed(&mut self) {
        self.tx.send(TestMessage::MarkFailed).await.unwrap();
    }
}

pub fn make_testharness(connection: Pool<Postgres>) -> (TestHarness, TestHarnessReceiver) {
    let (tx, rx) = futures::channel::mpsc::channel(100);
    (
        TestHarness { tx, connection },
        TestHarnessReceiver::new_from_rx(rx),
    )
}

pub enum TestMessage {
    MarkFailed,
}

pub struct TestHarnessReceiver {
    rx: Receiver<TestMessage>,
    should_clean: bool,
}

impl TestHarnessReceiver {
    pub fn new_from_rx(rx: Receiver<TestMessage>) -> Self {
        Self {
            rx,
            should_clean: true,
        }
    }

    pub async fn listen(&mut self) {
        while let Some(msg) = self.rx.next().await {
            match msg {
                TestMessage::MarkFailed => {
                    self.should_clean = false;
                }
            }
        }
    }

    pub fn clean(self) {
        if self.should_clean {
            // for folder_to_clean_up in self.temp_folders {
            //     debug!(
            //         "Removing temporary directory {}",
            //         folder_to_clean_up.display()
            //     );
            //     if let Err(e) = std::fs::remove_dir_all(&folder_to_clean_up) {
            //         warn!(
            //             "Could not clean up temporary folder '{}', it is lingering now: {e}",
            //             folder_to_clean_up.display()
            //         );
            //     }
            // }
        }
    }
}

inventory::collect!(IntegrationTestCase);
