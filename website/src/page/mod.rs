pub mod index;
pub mod user;

#[derive(Clone)]
pub struct WebData<'a> {
    pub title: &'a str,
    pub visitors: u64,
}
