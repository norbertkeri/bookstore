use color_eyre::eyre::bail;

use super::IntegrationTestCase;

pub enum TestSelector {
    Specific(&'static IntegrationTestCase),
    All(Box<dyn Iterator<Item = &'static IntegrationTestCase>>),
}

pub fn create_test_selector() -> color_eyre::Result<TestSelector> {
    let mut iter = inventory::iter::<IntegrationTestCase>.into_iter();
    if let Some(filter_name) = std::env::args().nth(1) {
        let test = iter.find(|&case| case.name.starts_with(&filter_name));
        if let Some(test) = test {
            return Ok(TestSelector::Specific(test));
        }
        bail!("No tests found beginning with name {filter_name}");
    } else {
        Ok(TestSelector::All(Box::new(iter)))
    }
}
