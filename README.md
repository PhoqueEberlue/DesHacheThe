# DèsHachesThé

DèsHachesThé 🎲🪓🫖 is a DHT school project among Universitatea de Vest din Timisoara.

Authors:

Andrew Mary Huet de Barochez
Antoine David


## Architecture

![example-wasm](https://github.com/PhoqueEberlue/DesHacheThe/blob/main/project_architecture.svg)

We can identify 4 groups of entities: 
- Client using web browser
- The server running with flask, exposing an API and some routes for our application
- The database, that can be run on another machine. Here we are using BaseX
- The Kademlia network that is run on multiple machines

Kademlia machines uses Stax parsers in order to parse system informations of the machines into XML.
Those informations are then sent every 10 seconds to BaseX using its Rest api and XQuery.

The server has two roles: 
- interacting with Kademlia network
- providing client web service

For the first one, it owns routes in order to interact with the network, for example starting it or killing it.
To perform those operation the server get the network configuration.xml file stored in BaseX, then parse it with a Event based parser, and finally launches the docker containers.

The second role exposes a web service with routes for the user to display the content of the database with some filters.
The server get the data from BaseX using the python client and then performs XSLT transformations with XPath before sending it to the client.
The client can also trigger network starting and killing talked in the previous paragraph.

XForms files have also been implemented, yet they haven't been integrated to the project in time to permit CRUDS.

## Running the project

Dependencies:
- BaseX: https://docs.basex.org/wiki/Startup
- Python: https://www.python.org/downloads/
- Rustc: https://www.rust-lang.org/tools/install or
- Docker: https://docs.docker.com/engine/install/

### BaseX setup

After downloading BaseX simply run the server, we used the default ip and ports.

### Flask Server setup

Create a virtual env

```sh
python3 -m venv venv
source venv/bin/activate
```

Install requirements in node_launcher/
```sh
pip install -r requirements.txt
```

Running the server
```sh 
flask --app src/main.py run
```

### Kademlia setup 

This part requires the rust compiler with cargo if you want to test the nodes withouth docker.

In dht_core simply do the following to run a node
```sh
cargo run -- --secret-key-seed 40 
```

To build it with docker and let the client control the launch of containers, do the following in dht_core/:

```sh
docker build -t dht-core .
```
