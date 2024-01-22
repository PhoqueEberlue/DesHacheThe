import os
import xml.etree.ElementTree as ET
import docker
from flask import Flask, render_template
from flask import request, jsonify, current_app
from lxml import etree
from BaseXClient import BaseXClient

PREFIX = "dht"
DATABASE_NAME = "dht_db"
# DOCKER_CLIENT = docker.from_env()

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


# XForms and XSLT Part :

@app.route('/get_xml_count')
def get_xml_count():
    session = BaseXClient.Session('localhost', 1984, 'admin', 'admin')
    try:
        # BaseX Connection
        session.execute("OPEN dht_db")

        # Request to count the XML Files
        query = ('xquery count(collection("' + DATABASE_NAME
                 + '") except collection("' + DATABASE_NAME
                 + '/config.xml''"))')
        count = session.execute(query)
        return jsonify(int(count))

    except Exception as e:
        app.logger.error("Error while counting XML files: " + str(e))
        return jsonify(error=str(e))

    finally:
        session.close()


@app.route("/home")
def home():
    app.logger.info("HOME")
    return render_template('home.html')


def id_to_key(id):
    # TODO Here Logic to convert the id into the XML filename
    try:
        id = int(id)
    except ValueError as e:
        app.logger.error(str(e))
    switch_dict = {
        1: "12D3KooW9wXJxh5HwMaDyuvVbdvgnTW4dSXYu7BfDXXey1u5KD5L",
        2: "12D3KooWEzz3mFsESoRPa7uojryc2BKYkZWJqBKhhb6TLwSKEoJc",
        3: "12D3KooWP6FGhHBCCSHePPBoKtaEdvt5oCqMSdFXi9JQUztUQtvi"
    }

    # Utilisation de get() pour obtenir la valeur par défaut si la clé n'est pas présente
    result = switch_dict.get(id, "The id doesn't exist")
    app.logger.info("id = " + str(id))
    app.logger.info("result = " + str(result))
    return result


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


@app.route("/create_machine")
def create_machine():
    # TODO
    return


@app.route("/update_machine")
def update_machine():
    # TODO
    return


@app.route("/delete_machine")
def delete_machine():
    # TODO
    return


@app.route("/select_data", methods=["POST"])
def select_data():
    # Get JSON data from the request
    return choice_menu(request.get_json())


def choice_menu(data):
    menu1 = data.get("menu1")
    menu2 = data.get("menu2")
    menu3 = data.get("menu3")
    app.logger.info("menu1 :" + menu1 + "\nmenu2 :" + menu2 + "\nmenu3 :" + menu3)
    if int(menu1) == 0:
        return add_new_machine()
    elif int(menu2) != 1:
        return crud_machine()
    else:
        return general_view(data, menu3)


def add_new_machine():
    return 0


def crud_machine():
    return 1


def general_view(data, menu3):
    xml_key = id_to_key(data.get("menu1"))
    # BaseX Connection
    session = BaseXClient.Session('localhost', 1984, 'admin', 'admin')
    try:
        session.execute("OPEN dht_db")
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
        app.logger.info("ERROR : " + str(e))
        return jsonify({"error": str(e)})
    finally:
        session.close()


# Machine Constructor
class Machine:
    def __init__(self, seed, datas):
        self.seed = seed
        self.datas = datas

    def __repr__(self):
        return f"{self.seed}, {self.datas}"
