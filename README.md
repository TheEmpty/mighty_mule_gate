# Might Mule Gate (API)

## What
Uses the option ports on the Mighty Mule to provide a JSON API
to control the state of the Might Mule powered gate. This will
run an a raspberry pi using relays to connect the COM to controls.

## Why
USPS refuses to leave my packages at the gate like every other carrier.
By setting up this API, I can have another service that watches my USPS packages
and opens the gate when packages are out for delivery. This can also be
expanded to non-salty uses-cases like using an RFID to recognize cars and
having that call out to this micro service.

## How
Will update after feature complete* with photos of install and general idea on setup.
Right now only geared at developers that are comfortable to `git pull`
and `cargo build`.

## API Documentation

### `GET /gate`
Returns the current state of the gate (polled via GPIO).
An array of lock expirations and the intendend lock state (may differ than actual state).

Ex:
```
{
    "state": "CLOSED",
    "locked_state": null,
    "locks": []
}
```

Or with locks,
```
{
    "state": "MOVING",
    "locked_state": "OPEN",
    "locks": [
        {
            "expires": {
                "secs": 1590568944,
                "nanos": 448115871
            }
        }
    ]
}
```

### `POST /gate`
Currently only allows the the `state` param to be passed in. The service
will then make a best-effort attempt to put the gate into the passed state.
It returns the same structure as `GET /gate` as a response on success.
If the gate is locked, or the request was malformed, the response will include
an "error" field with a human readable message.

### `POST /lock`
Takes in parameters `lock_state` and `lock_state_ttl_seconds`. Lock state allows
you to specify what state you want the get held in. The lock is then valid until
the TTL (time-to-live) or the lock is `DELETE`d. If the request can not be fulfilled,
a response with an "error" field is returned. The error field will contain a human
readable message. On success, the API returns a UUID that the caller can use to
`DELETE` the lock before the TTL expires. The UUID is not available via the API afterward.

Example success,
```
{
    "id": "9556daa0-3e24-410f-b848-e32f0adc4d15"
}
```

Example failure,
```
{
    "error": "Being held in OPEN state. Can not change to holding CLOSED."
}
```

### `DELETE /lock`
Takes in a single parameter `id` that was given via `POST /lock`.
Returns a single field, success, as a boolean. Eg,

```
{
	"success": true
}
```

### TODO:
* test suite
* possibly: a simple HTML page that has buttons to manually invoke API(s).
* logging
* cleanup code closer to feature complete
