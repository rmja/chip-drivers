pub struct Apn<'a> {
    pub apn: &'a str,
    pub username: &'a str,
    pub password: &'a str,
}

impl<'a> Apn<'a> {
    pub const fn new(apn: &'a str) -> Self {
        Self {
            apn,
            username: "",
            password: "",
        }
    }
}

impl<'a> From<&'a str> for Apn<'a> {
    fn from(value: &'a str) -> Self {
        Apn::new(value)
    }
}
