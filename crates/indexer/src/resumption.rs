use std::{
    fs::File,
    io::{BufReader, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub(crate) change_id: String,
    pub(crate) next_change_id: String,
}

pub struct StateWrapper<'a> {
    pub(crate) inner: Option<State>,
    pub(crate) path: &'a dyn AsRef<Path>,
}

impl<'a> StateWrapper<'a> {
    pub fn load_from_file(path: &'a dyn AsRef<Path>) -> Self {
        let inner = {
            if path.as_ref().exists() {
                let reader = BufReader::new(File::open(path.as_ref()).unwrap());
                serde_json::from_reader(reader).unwrap()
            } else {
                None
            }
        };

        Self { inner, path }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut f = File::create(self.path.as_ref())?;
        let serialized = serde_json::to_vec_pretty(&self.inner)?;
        f.write_all(&serialized)?;

        Ok(())
    }

    pub fn update(&mut self, s: State) {
        self.inner.replace(s);
    }
}
