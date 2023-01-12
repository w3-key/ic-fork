use crate::public_key_store::PublicKeyAddError;
use crate::public_key_store::PublicKeyGenerationTimestamps;
use crate::public_key_store::PublicKeySetOnceError;
use crate::public_key_store::PublicKeyStore;
use ic_protobuf::registry::crypto::v1::PublicKey;
use ic_protobuf::registry::crypto::v1::X509PublicKeyCert;
use mockall::predicate::*;
use mockall::*;

mock! {
    /// Mock PublicKeyStore object for testing interactions
    pub PublicKeyStore {}

    pub trait PublicKeyStore {
        fn set_once_node_signing_pubkey(
            &mut self,
            key: PublicKey,
        ) -> Result<(), PublicKeySetOnceError>;

        fn node_signing_pubkey(&self) -> Option<PublicKey>;

        fn set_once_committee_signing_pubkey(
            &mut self,
            key: PublicKey,
        ) -> Result<(), PublicKeySetOnceError>;

        fn committee_signing_pubkey(&self) -> Option<PublicKey>;

        fn set_once_ni_dkg_dealing_encryption_pubkey(
            &mut self,
            key: PublicKey,
        ) -> Result<(), PublicKeySetOnceError>;

        fn ni_dkg_dealing_encryption_pubkey(&self) -> Option<PublicKey>;

        fn set_once_tls_certificate(
            &mut self,
            cert: X509PublicKeyCert,
        ) -> Result<(), PublicKeySetOnceError>;

        fn tls_certificate<'a>(&'a self) -> Option<&'a X509PublicKeyCert>;

        fn add_idkg_dealing_encryption_pubkey(&mut self, key: PublicKey) -> Result<(), PublicKeyAddError>;

        fn set_idkg_dealing_encryption_pubkeys(
            &mut self,
            keys: Vec<PublicKey>,
        ) -> Result<(), std::io::Error>;

        fn idkg_dealing_encryption_pubkeys(&self) -> Vec<PublicKey>;

        fn generation_timestamps(&self) -> PublicKeyGenerationTimestamps;
        }
}
