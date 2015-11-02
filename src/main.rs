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

#[macro_use]
extern crate clap;
extern crate yaml_rust;

mod database_config;

use clap::App;

use std::io::prelude::*;
use std::{env, process, error};
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;

const DEFAULT_PROVISIONING_PATH: &'static str = "/home/vagrant/workspace/bankfacil/provisioning";

fn main() {
    let matches = App::new("bkf-mysql")
        .version(&crate_version!())
        .author("Renato Zannon <renato@bankfacil.com.br>")
        .about("Loads database configuration from the provisioning project and spawns mysql")
        .arg_from_usage("<project> 'The name of the project to connect'")
        .args_from_usage(
            "-p --prod    'Connects to the production environment (default)'
             -s --staging 'Connects to the staging environment'")
        .after_help(&format!("The path to the provisioning project is expected to be {}.\n\
                              Set the BKF_PROVISIONING_PATH environment variable to override it.\n\n\
                              To use an alternative mysql executable (e.g. mycli), set the \
                              MYSQL_EXECUTABLE environment variable", DEFAULT_PROVISIONING_PATH))
        .get_matches();

    let project = matches.value_of("project").unwrap();
    let environment = if matches.is_present("staging") { "staging" } else { "production" };

    let provisioning_path = env::var("BKF_PROVISIONING_PATH").map(Cow::Owned).unwrap_or(Cow::Borrowed(DEFAULT_PROVISIONING_PATH));

    let config_file_path = Path::new(&*provisioning_path)
        .join("application/roles/app/files/deploy/")
        .join(project)
        .join(environment)
        .join("config/database.yml");

    let cfg = match read_config_from_file(&config_file_path, environment) {
        Ok(cfg) => cfg,
        Err(err) => {
            println!("Error reading config file {}:\n\t{}", config_file_path.display(), err);
            process::exit(1);
        }
    };

    spawn_mysql(cfg);
}

fn read_config_from_file<P: AsRef<Path>>(path: P, environment: &'static str) -> Result<database_config::DatabaseConfig, Box<error::Error>> {
    let config_file = try!(File::open(&path));

    let mut config_contents = String::new();
    try!(BufReader::new(config_file).read_to_string(&mut config_contents));

    database_config::parse(config_contents, environment).map_err(From::from)
}

fn spawn_mysql(cfg: database_config::DatabaseConfig) {
    let mysql_exec = env::var("MYSQL_EXECUTABLE").map(Cow::Owned).unwrap_or(Cow::Borrowed("mysql"));

    process::Command::new(&*mysql_exec)
        .args(&[
          "--host",     &cfg.host,
          "--port",     &cfg.port.to_string(),
          "--user",     &cfg.username,
          "--database", &cfg.database])
        .arg(format!("--password={}", cfg.password))
        .spawn()
        .and_then(|mut child| child.wait())
        .unwrap();
}
