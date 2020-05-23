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

### TODO:
* test suite
* possibly: a simple HTML page that has buttons to open, close, or hold gate.
* logging
* Add physical switches to the gate so the state is more accurate.
* Add logic for setting relay pins high/low to actually move the gate instead of just logging
* cleanup code closer to feature complete
