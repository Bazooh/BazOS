#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    path: String,
}

impl Path {
    pub fn new(mut path: String) -> Self {
        if path.ends_with("/") {
            path.truncate(path.len() - 1);
        }
        Path { path }
    }

    pub fn from_parts(parts: &[&str]) -> Self {
        Path::new(parts.join("/"))
    }

    pub fn split(&self) -> Vec<&str> {
        self.path.split("/").collect()
    }

    pub fn add(&self, name: String) -> Self {
        if name.starts_with("/") {
            Path::new(format!("{}{}", self.path, name))
        } else {
            Path::new(format!("{}/{}", self.path, name))
        }
    }
}
