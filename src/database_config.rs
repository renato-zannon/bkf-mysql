/*  bkf-mysql - Convenience utility for connecting to mysql databases
 *  Copyright © 2015 Renato Zannon
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>. */

use yaml_rust::{self, YamlLoader, Yaml};
use std::{error, fmt};

#[derive(Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug)]
pub enum Error {
    YamlError(yaml_rust::ScanError),
    EnvironmentNotFound(&'static str),
    KeyMissing { key: &'static str, env: &'static str }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::EnvironmentNotFound(env) => write!(fmt, "Environment {} not present", env),
            Error::KeyMissing { key, env }  => write!(fmt, "Required key '{}' not found for environment '{}'", key, env),
            Error::YamlError(ref err)       => write!(fmt, "YAML parsing error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EnvironmentNotFound(_) => "Requested environment not found on the YAML file",
            Error::KeyMissing { .. }      => "Required key not found for environment",
            Error::YamlError(_)           => "YAML parsing error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::EnvironmentNotFound(_) => None,
            Error::KeyMissing { .. }      => None,
            Error::YamlError(ref err)     => Some(err),
        }
    }
}

impl From<yaml_rust::ScanError> for Error {
    fn from(err: yaml_rust::ScanError) -> Error {
        Error::YamlError(err)
    }
}

pub fn parse<S: AsRef<str>>(s: S, env: &'static str) -> Result<DatabaseConfig, Error> {
    let doc = &try!(YamlLoader::load_from_str(s.as_ref()))[0];
    let production_config = &doc[env];

    if production_config == &Yaml::BadValue {
        return Err(Error::EnvironmentNotFound(env));
    }

    let required_key = |name: &'static str| {
        match production_config[name].as_str() {
            Some(value) => Ok(value.to_owned()),
            None        => Err(Error::KeyMissing { env: env, key: name }),
        }
    };

    Ok(DatabaseConfig {
        host: try!(required_key("host")),
        port: production_config["port"].as_i64().unwrap_or(3306) as u16,
        username: try!(required_key("username")),
        password: try!(required_key("password")),
        database: try!(required_key("database")),
    })
}
