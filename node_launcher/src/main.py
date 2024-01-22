import os
import xml.etree.ElementTree as ET
import docker
from flask import Flask, render_template, redirect, url_for
from flask import request, jsonify, current_app
from lxml import etree
from BaseXClient import BaseXClient

DOCKER_PREFIX = "dht"

DATABASE_NAME = "dht-db"
DATABASE_PORT = 1984
DATABASE_USERNAME = "admin"
DATABASE_PASSWORD = "test"

DOCKER_CLIENT = docker.from_env()

app = Flask(__name__)


# -------------------- REST ROUTES --------------------
@app.route("/rest/run_all")
def run_all_containers():
    session = BaseXClient.Session(
        'localhost', DATABASE_PORT, DATABASE_USERNAME, DATABASE_PASSWORD)

    session.execute(f"OPEN {DATABASE_NAME}")

    xml_content = session.execute(f"GET config.xml")

    machines = parse_xml_config(xml_content)

    run_containers(machines)
    return "ok"


@app.route("/rest/kill_all")
def kill_all_containers():
    delete_all_containers()
    return "ok"


@app.route("/rest/create_machine")
def create_machine():
    # TODO
    return


@app.route("/rest/update_machine")
def update_machine():
    # TODO
    return


@app.route("/rest/delete_machine/<int:seed>")
def delete_machine(seed):
    session = BaseXClient.Session(
        'localhost', DATABASE_PORT, DATABASE_USERNAME, DATABASE_PASSWORD)

    session.execute(f"OPEN {DATABASE_NAME}")

    input = f"delete //machine[@seed eq {seed}]"

    query = session.query(input)
    query.execute()

    session.close()

    return redirect(url_for('home'))


# -------------------- VIEW ROUTES --------------------
# XForms and XSLT Part :

@app.route("/home")
def home():
    app.logger.info("HOME")

    session = BaseXClient.Session(
        'localhost', DATABASE_PORT, DATABASE_USERNAME, DATABASE_PASSWORD)

    result = session.execute(f"LIST {DATABASE_NAME}")

    xml_documents = {}
    index = 1
    for data in result.replace('\n', ' ').split(' '):
        if data.endswith(".xml") and not data == "config.xml":
            xml_documents[index] = data.replace(".xml", "")
            index += 1

    return render_template('home.html', xml_documents=xml_documents)


def apply_xslt(xml_content, xslt_path):
    try:
        # Load XML and XSLT
        xml_content = f"<records>{xml_content}</records>"
        xml_doc = etree.fromstring(xml_content)
        xslt_doc = etree.parse(os.path.join(app.root_path, xslt_path))

        # Apply transformation
        transform = etree.XSLT(xslt_doc)
        result_tree = transform(xml_doc)

        # Get the result
        transformed_result = str(result_tree)
        app.logger.info("LE RESULTAT:\n" + transformed_result)
        return transformed_result
    except Exception as e:
        raise RuntimeError(f"XSLT Transformation Error: {str(e)}")


@app.route("/select_data", methods=["POST"])
def select_data():
    # Get JSON data from the request
    data = request.get_json()

    machine_seed = data.get("menu1")
    menu2 = int(data.get("menu2"))
    menu3 = data.get("menu3")

    if menu2 == 1:
        return general_view(data, menu3)
    elif menu2 == 2:
        pass
    elif menu2 == 3:
        return redirect(url_for('delete_machine', seed=machine_seed))


def general_view(data, menu3):
    xml_key = data.get("menu1")
    # BaseX Connection
    session = BaseXClient.Session(
        'localhost', DATABASE_PORT, DATABASE_USERNAME, DATABASE_PASSWORD)

    try:
        session.execute(f"OPEN {DATABASE_NAME}")
        xquery = (
            f"xquery \n"
            f"for $record in doc('{DATABASE_NAME}/{xml_key}.xml')//record[position() le {menu3}]\n"
            f"return $record"
        )
        # f"xquery doc('{DATABASE_NAME}/{xml_key}.xml')"
        # Construct query with XML File name
        result = session.execute(xquery)

        # Apply XSLT Transform
        xslt_path = "static/xslt/example.xslt"
        transformed_result = apply_xslt(result, xslt_path)

        return jsonify({"result": transformed_result})
    except Exception as e:
        print(e)
        app.logger.info("ERROR : " + str(e))
        return jsonify({"error": str(e)})
    finally:
        session.close()


# Docker related functions

# Machine Constructor
class Machine:
    def __init__(self, seed, datas):
        self.seed = seed
        self.datas = datas

    def __repr__(self):
        return f"{self.seed}, {self.datas}"


def run_containers(machines: list['Machine']):
    for machine in machines:
        machine_name = DOCKER_PREFIX + machine.seed

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
        if container.name.startswith(DOCKER_PREFIX):
            if container.status == "running":
                container.stop()
                print(f"Container {container.name} stopped")

            container.remove()
            print(f"Container {container.name} deleted")


def parse_xml_config(xml_content) -> list['Machine']:
    tree = ET.ElementTree(ET.fromstring(xml_content))
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

