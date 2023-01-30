pub struct ApnInfo<'a> {
    pub apn: &'a str,
    pub username: &'a str,
    pub password: &'a str,
}

impl<'a> ApnInfo<'a> {
    pub const fn new(apn: &'a str) -> Self {
        Self {
            apn,
            username: "",
            password: "",
        }
    }
}
