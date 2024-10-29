# Rust RTE France

This is an api around the French TSO [RTE](https://rte-france.com).

The main endpoints [live here](https://data.rte-france.com).


## Provided APIs


### Generation forecast API

### Consumption forecast API

## Authentication

To use the API, you need to register an account in RTE's system (free).

Then you need to create an application. To do that,

- Go to [this application](https://data.rte-france.com/catalog/-/api/consumption/Consumption/v1.2)
- Click on "Abonnez vous à l'API"
- Fill the "Créer une application" form
- Then go to you application [here](https://data.rte-france.com/group/guest/apps)
- And copy the Client ID and Client Secret

These two variables need to be set as environment values:

- `CLIENT_ID`
- `CLIENT_SECRET`

