### TODO:
* a real readme
* 'hold state', probably with a TTL to prevent a dead service from never freeing. (use case: gate is open when packages are out for delivery- another microservice will tell this application to hold the gate open until the package is delivered)
* move gate configuration to a JSON file
* test suite
* port configuration to JSON
* handle error cases in API (unwrap fails)
* possibly: a simple HTML page that has buttons to open, close, or hold gate.
* better logging
* Add physical switches to the gate so the state is more accurate.
* Add logic for setting relay pins high/low to actually move the gate instead of just logging
