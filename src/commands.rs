pub enum Command {
    Tp(String),
}

impl Command {
    pub fn from_str(s: &str) -> Result<Self, CommandError> {
        let split = s.split(' ').collect::<Vec<&str>>();

        if let Some(cmd) = split.first() {
            match *cmd {
                "tp" => {
                    if let Some(player) = split.get(1) {
                        if split.get(2).is_none() {
                            Ok(Self::Tp(player.to_string()))
                        } else {
                            Err(CommandError::TooManyArguments)
                        }
                    } else {
                        Err(CommandError::NotEnoughArguments)
                    }
                }
                _ => Err(CommandError::CommandNotKnown),
            }
        } else {
            Err(CommandError::NoCommand)
        }
    }
}

pub enum CommandError {
    NoCommand,
    CommandNotKnown,
    TooManyArguments,
    NotEnoughArguments,
}
