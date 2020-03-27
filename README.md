# portfolio app

A rust implementation for a portfolio application.

## google cloud api, like store (bucket) or datastore

### authentication 
- api-key:
  - input: generate by gcloud
  - output: api-key
- jwt:
  - input: generate private key by gcloud, read from environment variable: PRIVATE_KEY
  - rest-call for get token
  - output: token

### rest api call
- input: api-key, token
- api-call:
  - get all buckets
  - datastore
