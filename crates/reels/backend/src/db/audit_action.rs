//! Type-safe enumeration for audit log action types.
//!
//! This enum ensures audit actions are consistent and prevents typos in action type strings.
//! All administrative actions that should be logged must be defined here.
//! Used throughout the audit logging system to ensure type safety and consistency.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    /// Organization was created by an admin
    CreateOrganization,
    
    /// Organization details were updated by an admin
    UpdateOrganization,
    
    /// Organization name was changed by an admin
    UpdateOrganizationName,
    
    /// Organization owner was changed by an admin
    ChangeOrganizationOwner,
    
    /// Organization credits were updated by an admin
    UpdateOrganizationCredits,
    
    /// Members were added to an organization in batch
    AddMembersBatch,
    
    /// User was created by an admin
    CreateUser,
    
    /// Multiple users were created in batch
    CreateUsersBatch,
    
    /// User was updated by an admin
    UpdateUser,
    
    /// User status was updated by an admin
    UpdateUserStatus,
    
    /// User credits (personal organization) were updated by an admin
    UpdateUserCredits,
    
    /// User was deleted by an admin
    DeleteUser,
    
    /// Multiple users were deleted in batch
    DeleteUsersBatch,
    
    /// Users were added to an organization in batch
    AddUsersToOrganizationBatch,
    
    /// Admin accessed audit logs list
    ListAuditLogs,
    
    /// Unlimited access was granted to a user
    GrantUnlimitedAccess,
    
    /// Unlimited access was revoked from a user
    RevokeUnlimitedAccess,
}

impl AuditAction {
    /// Converts the enum to a string representation for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateOrganization => "CREATE_ORGANIZATION",
            Self::UpdateOrganization => "UPDATE_ORGANIZATION",
            Self::UpdateOrganizationName => "UPDATE_ORGANIZATION_NAME",
            Self::ChangeOrganizationOwner => "CHANGE_ORGANIZATION_OWNER",
            Self::UpdateOrganizationCredits => "UPDATE_ORGANIZATION_CREDITS",
            Self::AddMembersBatch => "ADD_MEMBERS_BATCH",
            Self::CreateUser => "CREATE_USER",
            Self::CreateUsersBatch => "CREATE_USERS_BATCH",
            Self::UpdateUser => "UPDATE_USER",
            Self::UpdateUserStatus => "UPDATE_USER_STATUS",
            Self::UpdateUserCredits => "UPDATE_USER_CREDITS",
            Self::DeleteUser => "DELETE_USER",
            Self::DeleteUsersBatch => "DELETE_USERS_BATCH",
            Self::AddUsersToOrganizationBatch => "ADD_USERS_TO_ORGANIZATION_BATCH",
            Self::ListAuditLogs => "LIST_AUDIT_LOGS",
            Self::GrantUnlimitedAccess => "GRANT_UNLIMITED_ACCESS",
            Self::RevokeUnlimitedAccess => "REVOKE_UNLIMITED_ACCESS",
        }
    }
    
    /// Attempts to parse an action string from the database into an enum variant.
    /// Returns None if the string doesn't match any known action type.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CREATE_ORGANIZATION" => Some(Self::CreateOrganization),
            "UPDATE_ORGANIZATION" => Some(Self::UpdateOrganization),
            "UPDATE_ORGANIZATION_NAME" => Some(Self::UpdateOrganizationName),
            "CHANGE_ORGANIZATION_OWNER" => Some(Self::ChangeOrganizationOwner),
            "UPDATE_ORGANIZATION_CREDITS" => Some(Self::UpdateOrganizationCredits),
            "ADD_MEMBERS_BATCH" => Some(Self::AddMembersBatch),
            "CREATE_USER" => Some(Self::CreateUser),
            "CREATE_USERS_BATCH" => Some(Self::CreateUsersBatch),
            "UPDATE_USER" => Some(Self::UpdateUser),
            "UPDATE_USER_STATUS" => Some(Self::UpdateUserStatus),
            "UPDATE_USER_CREDITS" => Some(Self::UpdateUserCredits),
            "DELETE_USER" => Some(Self::DeleteUser),
            "DELETE_USERS_BATCH" => Some(Self::DeleteUsersBatch),
            "ADD_USERS_TO_ORGANIZATION_BATCH" => Some(Self::AddUsersToOrganizationBatch),
            "LIST_AUDIT_LOGS" => Some(Self::ListAuditLogs),
            "GRANT_UNLIMITED_ACCESS" => Some(Self::GrantUnlimitedAccess),
            "REVOKE_UNLIMITED_ACCESS" => Some(Self::RevokeUnlimitedAccess),
            _ => None,
        }
    }
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

