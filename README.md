# Password Microservice

## Getting started

### Installation

```sh
cargo install --git https://github.com/ferristhecrab/atom-pw
```

### Running

#### Prerequisite
MongoDB running with [authentication set up](https://www.geeksforgeeks.org/how-to-enable-authentication-on-mongodb/);

```sh
CONFIG=/home/yourname/.config/atomics/pw.json atom-pw
```

Where `CONFIG` can be replaced with the location to the config file.

### Configuration

```jsonc
{
  "mongodb": {
    "address": "mongodb://localhost:27017",
    "username": "adminuser",        // your mongodb authenticated username
    "password": "LONG_STRING",      // your mongodb authenticated password
    "authDB": "admin",              // authentication db
    "pwDB": "atomics"               // password data will be stored in this db
                                    // under the "passwords" collection
  },
  "argon2": {
    "pepper": "A_DIFFERENT_LONG_STRING",    // secret for pepper
    "algorithm": "Argon2id",                // other options include: "Argon2i" and "Argon2d"
    "version": 19,                          // other options: 16
    "mCost": 19456,                         // memory size in 1KiB blocks, require >= 8*p_cost
    "tCost": 2,                             // number of iterations
    "pCost": 1,                             // degree of parallelism
    "outputLen": 32                         // length of password hash
  }
}
```

## API

Struct definitions can be found in [schema](./src/schema), exposed struct `Router` and `InternalRouter` in [router.rs](./src/router.rs) for squashed microservices.

Note that all endpoints may return an error variant
```jsonc
{
  "type": "error",
  "reason": "String"
}
```

### [POST] `/api/pw/v1/create`]

Creates a new password entry.

#### Request

```jsonc
{
  "pw": "String"   // password of entry
}
```

#### Response

```jsonc
{
  "type": "created",
  "id": "UInt"         // id of entry created
}
```

### [POST] `/api/pw/v1/check`

Check if a given password is correct.

#### Request

```jsonc
{
  "id": "UInt",        // id of entry to verify for
  "pw": "String"
}
```

#### Response

```jsonc
{
  "type": "checked",
  "match": "Bool"
}
```

### [POST] `/api/pw/v1/set`

Change value of an existing entry

#### Request

```jsonc
{
  "id": "UInt",
  "pw": "String"    // the new password
}
```

#### Response

```jsonc
{
  "type": "set"
}
```

### [POST] `/api/pw/v1/remove`

Remove an existing entry

#### Request

```jsonc
{
  "id": "UInt",
}
```

#### Response

```jsonc
{
  "type": "removed"
}
```
