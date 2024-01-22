document.addEventListener('DOMContentLoaded', function () {
    var menu1 = document.getElementById('menu1');
    var menu2 = document.getElementById('menu2');
    var menu3 = document.getElementById('menu3');
    initMenus(null)

    // Adding listener to changes of menu 1
    menu1.addEventListener('change', function () {
        if (menu1.value != '0') {
            initMenus(menu1.value);
        } else {
            menu2.style.display = 'none';
            menu3.style.display = 'none';
        }
    });

    // Adding listener to changes of menu 2
    menu2.addEventListener('change', function () {
        if (menu2.value === '2' || menu2.value === '3') {
            menu3.style.display = 'none';
        } else {
            menu3.style.display = 'block';
        }
    });
});


// Initialize menu visibility and values
function initMenus(value_menu_1) {
    if (value_menu_1 != null) {
        menu1.value = value_menu_1;
    } else {
        menu1.value = '1';
    }
    menu2.value = '1';
    menu3.value = '1';
    menu2.style.display = 'block';
    menu3.style.display = 'block';
};



function sendData() {
    var menu1Value = document.getElementById("menu1").value;
    var menu2Value = document.getElementById("menu2").value;
    var menu3Value = document.getElementById("menu3").value;

    // Créer un objet avec les valeurs à envoyer
    var data = {
        menu1: menu1Value,
        menu2: menu2Value,
        menu3: menu3Value
    };

    // Convertir l'objet en chaîne JSON
    var jsonData = JSON.stringify(data);

    // Envoyer la requête POST à la nouvelle route Flask
    fetch('/select_data', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: jsonData
    })
        .then(response => response.json())
        .then(data => {
            // Mettre le résultat dans la balise main
            document.querySelector('main').innerHTML = data.result;
        })
        .catch((error) => {
            console.error('Error:', error);
        });
}

// function menu1Creation(xmlCount) {
//     // Delete existing option inside menu 1
//     var menu1 = document.getElementById('menu1');
//     menu1.innerHTML = '';
// 
//     // Add option 0
//     var option0 = document.createElement('option');
//     option0.value = '0';
//     option0.text = 'Add a Machine';
//     menu1.appendChild(option0);
// 
//     // Add other options
//     for (var i = 1; i <= xmlCount; i++) {
//         var option = document.createElement('option');
//         option.value = i.toString();
//         option.text = 'Machine ' + i;
//         menu1.appendChild(option);
//     }
//     initMenus(null)
// }
