import xml.etree.ElementTree as ET
import subprocess
from flask import Flask

app = Flask(__name__)


@app.route("/")
def hello_world():
    return "<p>Hello, World!</p>"


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


def launch_machines(machines: list['Machine']):
    for machine in machines:
        command = f"docker run --network='host' \
        --env secret_key_seed={machine.seed} --datas "

        for key, value in machine.datas:
            command += f"{key}={value}"

        command += " dht-core:latest"

        subprocess.run(command)


class Machine:
    def __init__(self, seed, datas):
        self.seed = seed
        self.datas = datas

    def __repr__(self):
        return f"{self.seed}, {self.datas}"
