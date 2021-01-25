# VIT Servicing Station

--------------

VIT as a service (VaaS)

--------------


### Building tips and tricks

We use [`diesel`](http://diesel.rs/) for database (`sqlite3`) integration. Please refer to the [`diesel_cli` documentation](https://docs.rs/crate/diesel_cli/) to understand how to operate with migrations and setup.

Diesel generates rust code based on a *SQL* migration script (`/migrations/*/up.sql`) when running the migration with `diesel_cli`.
Diesel code generation is configured in the `diesel.toml` file. Right now it just contains the path on where the rust code should be generated.
Currently we use only one migration and make changes to it as needed. This is due to the fact that for each fund we spin up a new servicing station instance.

Another file to look at is the `.env` file. This file holds the environment variables used by this project sql configuration.
`diesel` uses a `DATABASE_URL` variable to know where should he generate the database file. 

### Server settings

The server settings can be loaded via three options, **environment variables**, **command line flags** and a **json file**.
These configurations follows some priority from low to high. 
Env variables are overwritten by command line flags if used and those before are overwritten by the json file if used too.

#### Env variables

- `DATABASE_URL` -> `URL` for the database connection
- `TLS_CERT_FILE` ->  Path to server X.509 certificate chain file, must be PEM-encoded and contain at least 1 item
- `TLS_PRIVATE_KEY_FILE` -> Path to server private key file, must be PKCS8 with single PEM-encoded, unencrypted key
- `CORS_ALLOWED_ORIGINS` -> Semicolon separated list of allowed `CORS` origins. For example: `https://foo.test;https://test.foo:5050`

#### Command line flags
The command line flags can be retrieved using the `--help` when running the server:

```bash
--address <address>                        Server binding address [default: 0.0.0.0:3030]
--allowed-origins <allowed-origins>        If none provided, echos request origin [env: CORS_ALLOWED_ORIGINS=]
--block0-path <block0-path>                block0 static file path [default: ./resources/v0/block0.bin]
--cert-file <cert-file>
    Path to server X.509 certificate chain file, must be PEM-encoded and contain at least 1 item [env:
    TLS_CERT_FILE=]
--db-url <db-url>                          Database url [env: DATABASE_URL=]  [default: ./db/database.sqlite3]
--in-settings-file <in-settings-file>      Load settings from file
--log-level <log-level>                    Application logging level
--log-output-path <log-output-path>        Output log file path
--max-age-secs <max-age-secs>              If none provided, CORS responses won't be cached
--out-settings-file <out-settings-file>    Dump current settings to file
--priv-key-file <priv-key-file>
    Path to server private key file, must be PKCS8 with single PEM-encoded, unencrypted key [env: TLS_PK_FILE=]
```

Some of the flags default to the environment variables explained above is not set.
Some of them have default values as fallback in case nor the env variable nor the flag is set.

#### JSON file configuration
Additionally if the you can load the whole configuration from a json file providing the path to the file within the `--in-settings-file`.
An example of the contents of the file would be like this:
```json
{
    "address" : "0.0.0.0:3030",
    "tls" : {
        "cert_file" : "./foo/bar.pem",
        "priv_key_file" : "./bar/foo.pem"
    },
    "cors" : {
        "allowed_origins" : ["https://foo.test", "https://test.foo"],
        "max_age_secs" : 60
    },
    "db_url": "./database.sqlite3",
    "block0_path": "./test/bin.test",
    "log" : {
        "log_output_path" : "./server.log",
        "log_level" : "error"    
    }
}
```

There is an option to dump a configuration into a `JSON` file with the `--out-settings-file` providing the path to the out file.
This option will dump the configuration with the defaults, already set environment variables or provided flags into the file.


## Playground

The VIT servicing station offers a **GraphQL** playground endpoint (`api/v0/graphql/playground`) designed to offer a user friendly approach to the API.

![playground](https://user-images.githubusercontent.com/1133113/86589565-2da0de00-bf8e-11ea-89e3-bbd19f5c29b4.png)

Some considerations when using the playground:

* There is no need of an `API-Token` for accessing it. But, the token is needed for any request performed by the playground.
In order to use your API-Token just add it inside the json configuration for headers in the bottom left part of the webpage. For example: `{ "API-Token" : "your api token here" }`
* Since the API-Token is needed for any request. Both the **docs** and the **schema** will not be available till the header is filled.
* Depending on the deployment and/or  your server configuration you may need to configure properly the **CORS** settings in order to make the playground work as expected.

## CLI

The `vit-servicing-station-cli` is an accompanying tool to interact with some of the ecosystem.
Right now it offers the following commands:

### api-token

#### generate
It is possible to generate api tokens (URL safe base64 encoded) with a simple command. For example:
```bash 
❯ ./vit-servicing-station-cli api-token generate
F-4QxU3FrbH7qg
```

It can be combined with two (optional) arguments:
* `--n` number of tokens to generate
* `--size` length (in **bytes**) of the tokens

#### add
We can add a token to some db using the tool too:

```bash
./vit-servicing-station-cli api-token add --db-url ../../db/vit_station_new.db --tokens 1CNDAo43fo4ktQ 0wNbdTDMJCFcnw
```

We need to provide the url to the database where we want it to be inserted (with `--db-url`) and the tokens we want too 
insert (with `--tokens` followed by the tokens). 
Notice that the token is in the same URL safe base64 encoding as we generated in the previous command.

**If not specifying** the `--tokens` argument the cli will read the input from the standard input the tokens we want to insert.
This enables the command to be piped from another command, for example:

```bash
./vit-servicing-station-cli api-token generate --size 10 --n 10 | ./vit-servicing-station-cli api-token add --db-url ../../db/vit_station_new.db
```
