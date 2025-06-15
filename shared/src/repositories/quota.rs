use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::models::UserDocument;
use crate::repositories::{users::UserRepository, vault::VaultRepository};

pub struct QuotaManager;

impl QuotaManager {
    fn plan_limits() -> HashMap<bool, (u32, u32)> {
        HashMap::from([
            (false, (5, 200)),   // Free: (secret_quota, request_quota)
            (true, (50, 5_000)), // Paid
        ])
    }

    pub fn get_quotas(user: &UserDocument) -> (u32, u32) {
        let plan = user.has_paid;
        Self::plan_limits().get(&plan).cloned().unwrap_or((0, 0))
    }

    pub async fn enforce_secret_quota(
        vault_repo: &VaultRepository,
        user: &UserDocument,
    ) -> Result<()> {
        let (secret_quota, _) = Self::get_quotas(user);
        let count = vault_repo.count_secrets_by_user(user.id).await?;

        if count >= secret_quota {
            return Err(anyhow!("You have reached your secret quota for this plan."));
        }
        Ok(())
    }

    pub async fn enforce_request_quota(
        user_repo: &UserRepository,
        user: &UserDocument,
    ) -> Result<()> {
        let (_, request_quota) = Self::get_quotas(user);

        let mut user_doc = user_repo
            .get_user_by_id(&user.id.to_string())
            .await?
            .ok_or_else(|| anyhow!("User not found while enforcing request quota."))?;

        if user_doc.request_quota == 0 {
            return Err(anyhow!(
                "You have exhausted your request quota for this month."
            ));
        }

        user_doc.request_quota -= 1;
        user_repo
            .update_request_quota(user_doc.id, user_doc.request_quota)
            .await?;
        Ok(())
    }

    pub async fn update_quota_on_secret_create(
        user_repo: &UserRepository,
        user: &UserDocument,
    ) -> Result<()> {
        let mut user_doc = user_repo
            .get_user_by_id(&user.id.to_string())
            .await?
            .ok_or_else(|| anyhow!("User not found when updating secret quota."))?;

        if user_doc.secret_quota == 0 {
            return Err(anyhow!("No more secret quota remaining."));
        }

        user_doc.secret_quota -= 1;
        user_repo
            .update_secret_quota(user_doc.id, user_doc.secret_quota)
            .await?;
        Ok(())
    }

    pub async fn update_quota_on_secret_delete(
        user_repo: &UserRepository,
        user: &UserDocument,
    ) -> Result<()> {
        let mut user_doc = user_repo
            .get_user_by_id(&user.id.to_string())
            .await?
            .ok_or_else(|| anyhow!("User not found when updating secret quota."))?;

        let (max_secret_quota, _) = Self::get_quotas(&user_doc);

        if user_doc.secret_quota < max_secret_quota {
            user_doc.secret_quota += 1;
            user_repo
                .update_secret_quota(user_doc.id, user_doc.secret_quota)
                .await?;
        }

        Ok(())
    }
}
