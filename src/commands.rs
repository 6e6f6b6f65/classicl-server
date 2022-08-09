pub enum Commands {
    Tp(String),
}

impl Commands {
    pub fn from_str(s: &str) -> Option<Self> {
        let split = s.split(' ').collect::<Vec<&str>>();

        if let Some(cmd) = split.get(0) {
            match *cmd {
                "tp" => {
                    if let Some(player) = split.get(1) {
                        Some(Self::Tp(player.to_string()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
