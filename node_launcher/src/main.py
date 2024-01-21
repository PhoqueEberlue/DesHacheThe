import xml.etree.ElementTree as ET
import docker
from flask import Flask

PREFIX = "dht"
DOCKER_CLIENT = docker.from_env()

app = Flask(__name__)


@app.route("/run_all")
def run_all():
    # TODO:
    # 1. fetch config stored in baseX
    # OR
    # 2. receive it as parameter
    # 1. looks better
    machines = parse_xml_config()

    run_containers(machines)
    return "ok"


@app.route("/delete_all")
def delete_all():
    delete_all_containers()
    return "ok"


def parse_xml_config() -> list['Machine']:
    tree = ET.parse('../node_launcher_config.xml')
    root = tree.getroot()

    machines = []

    for machine in root:
        seed = machine.attrib["seed"]
        datas = {}

        for data in machine:
            key = data.attrib["key"]
            data = data.text.replace(" ", "").replace("\n", "")
            datas[key] = data

        machines.append(Machine(seed, datas))

    return machines


def run_containers(machines: list['Machine']):
    for machine in machines:
        machine_name = PREFIX + machine.seed

        data = ""

        for key, value in machine.datas.items():
            data += f"{key}={value};"

        env = {"secret_key_seed": machine.seed, "data": data}

        DOCKER_CLIENT.containers.run(
            image="dht-core",
            detach=True,  # -d
            environment=env,
            name=machine_name,
            # ports={62500: 62500},
            network="host",
        )

        print(f"Container {machine_name} started")


def delete_all_containers():
    containers = DOCKER_CLIENT.containers.list(all=True)

    for container in containers:
        if container.name.startswith(PREFIX):
            if container.status == "running":
                container.stop()
                print(f"Container {container.name} stopped")

            container.remove()
            print(f"Container {container.name} deleted")


class Machine:
    def __init__(self, seed, datas):
        self.seed = seed
        self.datas = datas

    def __repr__(self):
        return f"{self.seed}, {self.datas}"
