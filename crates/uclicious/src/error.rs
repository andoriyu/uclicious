//! Low level library that could be returned by the parser.
//! Based on https://github.com/draft6/libucl-rs

use std::error::Error;
use std::fmt;

use libucl_bind::{ucl_error_t, ucl_schema_error_code};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UclErrorType {
    Ok,
    Syntax,
    Io,
    State,
    Nested,
    Macro,
    Internal,
    SSL,
    Other,
}

impl UclErrorType {
    pub fn from_code(num: i32, desc: String) -> UclError {
        match num {
            _ if num == ucl_error_t::UCL_EOK as i32 => UclError {
                code: UclErrorType::Ok,
                desc,
            },
            _ if num == ucl_error_t::UCL_ESYNTAX as i32 => UclError {
                code: UclErrorType::Syntax,
                desc,
            },
            _ if num == ucl_error_t::UCL_EIO as i32 => UclError {
                code: UclErrorType::Io,
                desc,
            },
            _ if num == ucl_error_t::UCL_ESTATE as i32 => UclError {
                code: UclErrorType::State,
                desc,
            },
            _ if num == ucl_error_t::UCL_ENESTED as i32 => UclError {
                code: UclErrorType::Nested,
                desc,
            },
            _ if num == ucl_error_t::UCL_EMACRO as i32 => UclError {
                code: UclErrorType::Macro,
                desc,
            },
            _ if num == ucl_error_t::UCL_EINTERNAL as i32 => UclError {
                code: UclErrorType::Internal,
                desc,
            },
            _ if num == ucl_error_t::UCL_ESSL as i32 => UclError {
                code: UclErrorType::SSL,
                desc,
            },
            _ => UclError {
                code: UclErrorType::Other,
                desc,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UclError {
    code: UclErrorType,
    desc: String,
}

impl fmt::Display for UclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl Error for UclError {
    fn description(&self) -> &str {
        self.desc.as_ref()
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl UclError {
    pub fn boxed(self) -> Box<UclError> {
        Box::new(self)
    }

    pub fn kind(&self) -> UclErrorType {
        self.code
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UclSchemaErrorType {
    Ok,
    TypeMismatch,
    InvalidSchema,
    MissingProperty,
    Constraint,
    MissingDependency,
    Other,
}

impl UclSchemaErrorType {
    pub fn from_code(num: i32, desc: String) -> UclSchemaError {
        match num {
            _ if num == ucl_schema_error_code::UCL_SCHEMA_OK as i32 => UclSchemaError {
                code: UclSchemaErrorType::Ok,
                desc,
            },
            _ if num == ucl_schema_error_code::UCL_SCHEMA_TYPE_MISMATCH as i32 => UclSchemaError {
                code: UclSchemaErrorType::TypeMismatch,
                desc,
            },
            _ if num == ucl_schema_error_code::UCL_SCHEMA_INVALID_SCHEMA as i32 => UclSchemaError {
                code: UclSchemaErrorType::InvalidSchema,
                desc,
            },
            _ if num == ucl_schema_error_code::UCL_SCHEMA_MISSING_PROPERTY as i32 => {
                UclSchemaError {
                    code: UclSchemaErrorType::MissingProperty,
                    desc,
                }
            }
            _ if num == ucl_schema_error_code::UCL_SCHEMA_CONSTRAINT as i32 => UclSchemaError {
                code: UclSchemaErrorType::Constraint,
                desc,
            },
            _ if num == ucl_schema_error_code::UCL_SCHEMA_MISSING_DEPENDENCY as i32 => {
                UclSchemaError {
                    code: UclSchemaErrorType::MissingDependency,
                    desc,
                }
            }
            _ => UclSchemaError {
                code: UclSchemaErrorType::Other,
                desc,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UclSchemaError {
    pub code: UclSchemaErrorType,
    pub desc: String,
}

impl fmt::Display for UclSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl Error for UclSchemaError {
    fn description(&self) -> &str {
        self.desc.as_ref()
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
