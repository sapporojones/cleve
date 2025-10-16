# CLEVE - a Command Line EVE utility

## Introduction

As a casual EVE Online player I commonly find myself wanting quick access to information about a player or system or information about what Thera wormholes are about and I don't want to have a million tabs open in my browser.  One day I got bored and wrote a bunch of EVE/EVEscout api consumers in python but running python in the command line is clunky.  I wanted a reason to learn the rust programming language at a basic level and decided my first project should be rewriting all my old stuff in one monolithic command line rust utility.

## Usage
**After downloading and unpacking, on first use you need to run cleve update to download and unpack the EVE SDE to your current directory.**

A list of commands is below, subject to change.

### help
This command shows all available commands with a brief explanation of what each does.
![](https://i.imgur.com/H4rasgm.png)

### travel
This command shows all current Thera and Turnur wormholes in table format (retrieved from eve-scout.com's api).
![](https://i.imgur.com/mcZO3VU.png)

### thera
This command shows all current Thera wormholes (retrieved from eve-scout.com's api).
![](https://i.imgur.com/iJYDQeA.png)

### turnur
This command shows all current Turnur wormholes (retrieved from eve-scout.com's api).
![](https://i.imgur.com/QexwM3X.png)

### incursions
This command shows where all the active incursions are in the game and what their status is.
![](https://i.imgur.com/C8f4jmZ.png)

### pilot
This command shows zkill and public information about a pilot.
![](https://i.imgur.com/w4bqpYU.png)

### system
This command shows information about an EVE Solar System such as npc kills, number of jumps, a list of most recent kills and their age, and more.
![](https://i.imgur.com/OPpCmYP.png)

### status
EVE Online server status.
![](https://i.imgur.com/7kE1bWR.png)

### timers
This command shows all EVE Strategic Hub timers in table format with information about when the timer is coming up and who is the defender (soon to be deprecated).
![](https://i.imgur.com/LKIE0wP.png)

### update
Downloads and unpacks the latest EVE Online SDE from fuzzworks to the current directory (WARNING: This takes a few minutes depending on your internet connection.).  This command needs to be run on first use of the program.
![](https://i.imgur.com/yCb8fcS.png)

## Build instructions
To build cleve for any OS:
- Install rust
- Clone this repo
- Download the latest sqlite EVE SDE from [Fuzzworks](https://www.fuzzwork.co.uk/dump/sqlite-latest.sqlite.bz2) 
- Decompress the SDE you downloaded to the root of the project directory
- Install the sqlx cli using cargo `cargo install sqlx-cli`
- Use sqlx to prepare and cache SDE database queries `cargo sqlx prepare`
- Build the project `cargo build --release`