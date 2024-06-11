#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "SCREAMING_SNAKE_CASE")
)]
pub enum ItemCategory {
    Login,
    Password,
    ApiCredential,
    Server,
    Database,
    CreditCard,
    Membership,
    Passport,
    SoftwareLicense,
    OutdoorLicense,
    SecureNode,
    WirelessRouter,
    BankAccount,
    DriverLicense,
    Identity,
    RewardProgram,
    Document,
    EmailAccount,
    SocialSecurityNumber,
    MedicalRecord,
    SshKey,
    Custom,
    #[cfg_attr(feature = "serde", serde(untagged))]
    Unrecognized(String),
}
