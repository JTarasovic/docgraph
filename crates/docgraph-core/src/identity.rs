use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EntityIdError {
    WrongPrefix { expected: String },
    EmptyLocalComponent,
    InvalidFirstCharacter { character: char, byte: usize },
    InvalidCharacter { character: char, byte: usize },
}

pub(crate) fn validate_entity_id(id: &str, entity_type: &str) -> Result<(), EntityIdError> {
    let prefix = format!("{entity_type}:");
    let local = id
        .strip_prefix(&prefix)
        .ok_or_else(|| EntityIdError::WrongPrefix {
            expected: prefix.clone(),
        })?;
    let mut characters = local.char_indices();
    let Some((offset, first)) = characters.next() else {
        return Err(EntityIdError::EmptyLocalComponent);
    };
    if !first.is_ascii_alphanumeric() {
        return Err(EntityIdError::InvalidFirstCharacter {
            character: first,
            byte: prefix.len() + offset,
        });
    }
    if let Some((offset, character)) =
        characters.find(|(_, character)| !is_entity_id_local_character(*character))
    {
        return Err(EntityIdError::InvalidCharacter {
            character,
            byte: prefix.len() + offset,
        });
    }
    Ok(())
}

fn is_entity_id_local_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | '~')
}

impl fmt::Display for EntityIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPrefix { expected } => {
                write!(
                    formatter,
                    "must start with the entity type prefix {expected:?}"
                )
            }
            Self::EmptyLocalComponent => formatter.write_str("has an empty local component"),
            Self::InvalidFirstCharacter { character, byte } => write!(
                formatter,
                "local component starts with disallowed character {character:?} at byte {byte}; expected an ASCII letter or digit"
            ),
            Self::InvalidCharacter { character, byte } => write!(
                formatter,
                "local component contains disallowed character {character:?} at byte {byte}; expected only RFC 3986 unreserved characters (ASCII letters, digits, '.', '_', '-', or '~')"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_portable_local_components() {
        for id in ["task:1", "task:Alpha-2_beta.v3~draft", "task:Z"] {
            assert_eq!(validate_entity_id(id, "task"), Ok(()));
        }
    }

    #[test]
    fn identifies_the_invalid_local_construct() {
        assert_eq!(
            validate_entity_id("task:", "task"),
            Err(EntityIdError::EmptyLocalComponent)
        );
        assert_eq!(
            validate_entity_id("task:.hidden", "task"),
            Err(EntityIdError::InvalidFirstCharacter {
                character: '.',
                byte: 5,
            })
        );
        assert_eq!(
            validate_entity_id("task:has space", "task"),
            Err(EntityIdError::InvalidCharacter {
                character: ' ',
                byte: 8,
            })
        );
        assert_eq!(
            validate_entity_id("wrong:1", "task"),
            Err(EntityIdError::WrongPrefix {
                expected: "task:".to_owned(),
            })
        );
    }
}
