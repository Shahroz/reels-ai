//! Credit consumption constants
//!
//! This module defines the credit consumption values for different operations
//! in the system. These constants are used by the credits guard middleware
//! to determine how many credits each operation consumes.

/// Credit consumption values for different operations
pub struct CreditsConsumption;

impl CreditsConsumption {
    /// Credit cost for retouching images
    /// 
    /// This operation consumes 1 credit per image or asset processed
    pub const RETOUCH_IMAGES: i32 = 1;

    /// Credit cost for generating creative content from bundle
    /// 
    /// This operation consumes 1 credit per generation
    pub const GENERATE_CREATIVE_FROM_BUNDLE: i32 = 1;

    /// Credit cost for generating creative content
    /// 
    /// This operation consumes 1 credit per generation
    pub const GENERATE_CREATIVE: i32 = 1;

    /// Credit cost for generating style
    /// 
    /// This operation consumes 1 credit per generation
    pub const GENERATE_STYLE: i32 = 1;

    /// Credit cost for Google search browse
    /// 
    /// This operation consumes 1 credit per search
    pub const GOOGLE_SEARCH_BROWSE: i32 = 1;

    /// Credit cost for Narrativ browse raw
    /// 
    /// This operation consumes 1 credit per browse
    pub const NARRATIV_BROWSE_RAW: i32 = 1;

    /// Credit cost for Narrativ browse with query
    /// 
    /// This operation consumes 1 credit per browse
    pub const NARRATIV_BROWSE_WITH_QUERY: i32 = 1;

    /// Credit cost for Narrativ search
    /// 
    /// This operation consumes 1 credit per search
    pub const NARRATIV_SEARCH: i32 = 1;

    /// Credit cost for property research
    /// 
    /// This operation consumes 1 credit per research
    pub const PROPERTY_RESEARCH: i32 = 1;

    /// Credit cost for quick enhance image
    /// 
    /// This operation consumes 1 credit per image enhancement
    pub const QUICK_ENHANCE_IMAGE: i32 = 1;

    /// Credit cost for vocal tour
    /// 
    /// This operation consumes 0 credit per tour
    pub const VOCAL_TOUR: i32 = 0;
}

/// Operation types for credit consumption
/// 
/// This enum represents different types of operations that consume credits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditOperation {
    /// Retouch images operation
    RetouchImages,
    /// Generate creative from bundle operation
    GenerateCreativeFromBundle,
    /// Generate creative operation
    GenerateCreative,
    /// Generate style operation
    GenerateStyle,
}

impl CreditOperation {
    /// Get the credit cost for this operation
    pub fn credits_changed(&self) -> i32 {
        match self {
            CreditOperation::RetouchImages => CreditsConsumption::RETOUCH_IMAGES,
            CreditOperation::GenerateCreativeFromBundle => CreditsConsumption::GENERATE_CREATIVE_FROM_BUNDLE,
            CreditOperation::GenerateCreative => CreditsConsumption::GENERATE_CREATIVE,
            CreditOperation::GenerateStyle => CreditsConsumption::GENERATE_STYLE,
        }
    }

    /// Get a human-readable description of the operation
    pub fn description(&self) -> &'static str {
        match self {
            CreditOperation::RetouchImages => "Retouch Images",
            CreditOperation::GenerateCreativeFromBundle => "Generate Creative From Bundle",
            CreditOperation::GenerateCreative => "Generate Creative",
            CreditOperation::GenerateStyle => "Generate Style",
        }
    }
}

/// Credit reward action types
/// 
/// These constants define the action types used in the credit reward system
#[derive(Debug, Clone, utoipa::ToSchema)]
pub struct CreditRewardActionTypes {
    /// Action type for uploading assets
    /// 
    /// Used to track progress towards upload asset rewards
    pub upload_assets: &'static str,
    
    /// Action type for enhancing assets
    /// 
    /// Used to track progress towards enhance asset rewards
    pub enhance_assets: &'static str,
}

impl CreditRewardActionTypes {
    /// Action type for uploading assets
    /// 
    /// Used to track progress towards upload asset rewards
    pub const UPLOAD_ASSETS: &'static str = "upload_assets";
    
    /// Action type for enhancing assets
    /// 
    /// Used to track progress towards enhance asset rewards
    pub const ENHANCE_ASSETS: &'static str = "enhance_assets";
}

/// Credits for free users
pub const FREE_CREDITS: i32 = 30;

/// Credit conversion rate: 1 credit = 10 cents
pub const CREDITS_TO_CENTS_RATIO: i32 = 10;

/// The cutoff date for old users
/// 
/// This date is used to determine if a user is an old user
/// and should be exempt from credit checks
pub const OLD_USER_CUTOFF_DATE: chrono::DateTime<chrono::Utc> = chrono::NaiveDate::from_ymd_opt(2025, 09, 30)
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_utc();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_costs() {
        assert_eq!(CreditsConsumption::RETOUCH_IMAGES, 1);
        assert_eq!(CreditsConsumption::GENERATE_CREATIVE_FROM_BUNDLE, 1);
        assert_eq!(CreditsConsumption::GENERATE_CREATIVE, 1);
        assert_eq!(CreditsConsumption::GENERATE_STYLE, 1);
    }

    #[test]
    fn test_credit_operation_costs() {
        assert_eq!(CreditOperation::RetouchImages.credits_changed(), 1);
        assert_eq!(CreditOperation::GenerateCreativeFromBundle.credits_changed(), 1);
        assert_eq!(CreditOperation::GenerateCreative.credits_changed(), 1);
        assert_eq!(CreditOperation::GenerateStyle.credits_changed(), 1);
    }

    #[test]
    fn test_credit_operation_descriptions() {
        assert_eq!(CreditOperation::RetouchImages.description(), "Retouch Images");
        assert_eq!(CreditOperation::GenerateCreativeFromBundle.description(), "Generate Creative From Bundle");
        assert_eq!(CreditOperation::GenerateCreative.description(), "Generate Creative");
        assert_eq!(CreditOperation::GenerateStyle.description(), "Generate Style");
    }
}
