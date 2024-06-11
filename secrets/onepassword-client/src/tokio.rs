use std::{process::Stdio, sync::Arc};

use appbiotic_api_secrets_onepassword::{
    Api, ApiVersion, Item, OnePassword, OnePasswordError, User,
};
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct OnePasswordClient {
    service_token: Arc<String>,
}

impl OnePasswordClient {
    pub fn new(service_token: String) -> Self {
        Self {
            service_token: Arc::new(service_token),
        }
    }
}

#[async_trait]
impl OnePassword for OnePasswordClient {
    async fn api_version(&self) -> Result<ApiVersion, OnePasswordError> {
        Ok(ApiVersion {
            api: Api::Cli,
            version: self
                .op_exec_text(["--version"], None)
                .await?
                .trim()
                .to_owned(),
        })
    }

    async fn item_get(&self, vault_id: String, item_id: String) -> Result<Item, OnePasswordError> {
        self.op_exec_json(
            [
                "item",
                "get",
                "--vault",
                vault_id.as_str(),
                item_id.as_str(),
            ],
            None,
        )
        .await
    }

    async fn user_get_me(&self) -> Result<User, OnePasswordError> {
        self.op_exec_json(["user", "get", "--me"], None).await
    }
}

impl OnePasswordClient {
    async fn op_exec_json<'a, T, I>(
        &self,
        args: I,
        stdin: Option<Vec<u8>>,
    ) -> Result<T, OnePasswordError>
    where
        T: serde::de::DeserializeOwned,
        I: IntoIterator<Item = &'a str>,
    {
        let stdout = self
            .op_exec_text(args.into_iter().chain(["--format", "json"]), stdin)
            .await?;
        let response = serde_json::from_str(&stdout).map_err(|err| {
            OnePasswordError::new_unknown(format!("Failed to convert response: {err}: {}", stdout))
        })?;
        Ok(response)
    }

    async fn op_exec_text<'a, I>(
        &self,
        args: I,
        stdin: Option<Vec<u8>>,
    ) -> Result<String, OnePasswordError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let stdin_option = match &stdin {
            Some(_) => Stdio::piped(),
            None => Stdio::null(),
        };

        let mut child = Command::new("op")
            .stdin(stdin_option)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("OP_SERVICE_ACCOUNT_TOKEN", self.service_token.as_str())
            .args(args)
            .spawn()
            .map_err(|err| {
                OnePasswordError::new_unknown(format!("Failed to spawn op process: {err}"))
            })?;

        if let Some(stdin) = stdin {
            let mut child_stdin = child.stdin.take().ok_or_else(|| {
                OnePasswordError::new_unknown("Failed to get stdin from OnePassword cli".to_owned())
            })?;
            child_stdin.write_all(&stdin).await.map_err(|err| {
                OnePasswordError::new_unknown(format!("Failed to write to op process stdin: {err}"))
            })?;
        }

        let output = child.wait_with_output().await.map_err(|err| {
            OnePasswordError::new_unknown(format!("Failed to get output for op command: {err}"))
        })?;
        let stdout = String::from_utf8(output.stdout).unwrap_or_default();
        let stderr = String::from_utf8(output.stderr).unwrap_or_default();

        if !output.status.success() {
            let exit_code = output
                .status
                .code()
                .map(|x| x.to_string())
                .unwrap_or("?".to_owned());
            return Err(OnePasswordError::new_unknown(format!(
                "op exec fail with code `{exit_code}`: {}: {}",
                stdout, stderr
            )));
        }

        Ok(stdout)
    }
}
