# Samply.Focus v0.13.1 2025-04-22

## Bugfixes
* Don't double encode cached cql and sql query results

## Minor changes
* remove exliquid query with aliquotes
* add exliquid query for samples with status available

# Samply.Focus v0.13.0 2025-04-17

## Major changes
* Results of SQL queries can now be cached
* Add query for organoid internal dashboard


# Samply.Focus v0.12.0 2025-03-14

## Minor changes
* Warning if blaze availability check has non-200 HTTP status
* Fix Dockerfile for ./dev/focusdev build and ENDPOINT_URL in README.md
* Add SQL queries for Exliquid


# Samply.Focus v0.11.0 2025-02-10

## Major changes
* Querying EUCAIM API v1
* CQL generation supports empty AST of an arbitary debth

## Minor changes
* Add SQL query for the public SIORGP dashboard for the NeoMatch project
* Fix and rename the SQL query for the public SIORGP dashboard for the MetPredict project


# Samply.Focus v0.10.0 2025-02-03

## Major changes
* Laplace-rs version 0.5.0 (includes a statrs breaking change)
* DKTK_REPLACE_SPECIMEN_STRATIFIER, DKTK_REPLACE_HISTOLOGY_STRATIFIER for sample centric search

## Minor changes
* Allow Zlib license


# Samply.Focus v0.9.0 2024-12-11

## Major changes
* EHDS2 query support

## Minor changes
* Separated exporter API key CLA from authorization header CLA


# Samply.Focus v0.8.0 2024-11-04

In this release, we are supporting 4 types of SQL queries for Exliquid and Organoids

## Major changes
* Allowlist of SQL queries


# Samply.Focus v0.7.0 2024-09-24

In this release, we are extending the supported data backends beyond CQL-enabled FHIR stores. We now support PostgreSQL as well. Usage instructions are included in the Readme.

## Major changes
* PostgreSQL support added

  

# Focus -- 2023-02-08

This is the initial release of Focus, a task distribution application designed for working with Samply.Beam. Currently, only Samply.Blaze is supported as an endpoint, but other endpoints can easily be integrated.
