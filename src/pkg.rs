pub(crate) struct Pkg {
    pub(crate) namespace: &'static str,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
}

impl Pkg {
    pub const fn new(namespace: &'static str, name: &'static str, version: &'static str) -> Self {
        Self {
            namespace,
            name,
            version,
        }
    }

    pub fn package_name(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }
}
