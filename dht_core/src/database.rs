use reqwest;

use std::error::Error;

static DB_NAME: &'static str = "dht-db";

pub(crate) struct DatabaseConnection {
    client: reqwest::Client,
    peer_id: String
}

impl DatabaseConnection {
    pub async fn new(peer_id: String) -> Result<DatabaseConnection, Box<dyn Error>> {
        let client = reqwest::Client::new();

        let body = format!("
            <command>
                <text>LIST {}</text>
            </command>", DB_NAME);

        let body = client
            .post(format!("http://127.0.0.1:8080/rest/"))
            .basic_auth("admin", Some("test"))
            .body(body)
            .send()
            .await?
            .text()
            .await?;

        if !body.contains(&peer_id) {
            let _ = client
                .get(format!("http://127.0.0.1:8080/rest/{}?command=ADD+TO+{}.xml+<sysinfo></sysinfo>", DB_NAME, peer_id))
                .basic_auth("admin", Some("test"))
                .send()
                .await?
                .text()
                .await?;
        } 

        Ok(DatabaseConnection { client, peer_id })
    }

    pub async fn add_sysinfo_record(&self, record: String) -> Result<(), Box<dyn Error>> {
        let body = format!(
                "<query>
                    <text>
                        insert node .//record as last into doc('{}/{}.xml')/sysinfo
                    </text>
                    <context>{}</context>
                 </query>", DB_NAME, self.peer_id, record);

        let _ = self.client.post("http://127.0.0.1:8080/rest/")
            .basic_auth("admin", Some("test"))
            .body(body)
            .send()
            .await?
            .text()
            .await?;

        Ok(())
    }
}
