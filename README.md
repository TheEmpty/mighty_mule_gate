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
* 'hold state', probably with a TTL to prevent a dead service from never freeing. (use case: gate is open when packages are out for delivery- another microservice will tell this application to hold the gate open until the package is delivered)
* test suite
* possibly: a simple HTML page that has buttons to open, close, or hold gate.
* better logging
* Add physical switches to the gate so the state is more accurate.
* Add logic for setting relay pins high/low to actually move the gate instead of just logging
* Security? Low priority since hacking the LAN and finding the IP and API is a lot harder than just hopping the fence. If they go through that effort, they can open the gate.
* cleanup code close to feature complete
