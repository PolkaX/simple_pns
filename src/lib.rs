#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::{
    storage,
    memory::vec::Vec,
};
use ink_lang::contract;

pub type Text = Vec<u8>;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    event Register {
        name: Hash,
        from: AccountId,
    }

    event RegisterAbi {
        from: AccountId,
        name: Hash,
        code_hash: Hash,
        abi: Text
    }

    event SetAddress {
        name: Hash,
        from: AccountId,
        old_address: Option<AccountId>,
        new_address: AccountId,
    }

    event Transfer {
        name: Hash,
        from: AccountId,
        old_owner: Option<AccountId>,
        new_owner: AccountId,
    } 

    struct SimplePns {
        /// A hashmap to store all name to addresses mapping
        name_to_address: storage::HashMap<Hash, AccountId>,
        /// A hashmap to store all name to owners mapping
        name_to_owner: storage::HashMap<Hash, AccountId>,
        name_to_abi: storage::HashMap<Hash, Text>,
        code_hash_to_abi: storage::HashMap<Hash, Text>,
        account_to_code_hash_list: storage::HashMap<AccountId, Vec<Hash>>,
        default_address: storage::Value<AccountId>,
    }

    impl Deploy for SimplePns {
        /// Initializes contract with default address.
        fn deploy(&mut self) {
            self.default_address.set(AccountId::from([0x0; 32]));
        }
    }

    impl SimplePns {
        /// Register specific name with caller as owner
        pub(external) fn register(&mut self, name: Hash, address: AccountId) -> bool {
            let caller = env.caller();
            if self.is_name_exist_impl(name) {
                return false
            }
            // env.println(&format!("register name: {:?}, owner: {:?}", name, caller));
            self.name_to_owner.insert(name, caller);
            self.name_to_address.insert(name, address);
            env.emit(Register {
                name: name,
                from: caller,
            });
            true
        }

        /// Register abi with name and code_hash
        pub(external) fn register_abi(&mut self, name: Hash, code_hash: Hash, abi: Text) -> bool {
            let caller = env.caller();
            if self.is_name_exist_impl(name) {
                return false
            }
            // env.println(&format!("register_abi name: {:?}, owner: {:?}", name, caller));
            self.name_to_abi.insert(name, abi.clone());
            self.code_hash_to_abi.insert(code_hash, abi.clone());
            match self.account_to_code_hash_list.get_mut(&caller) {
                None => {
                    let mut new_vec = Vec::new();
                    new_vec.push(code_hash);
                    self.account_to_code_hash_list.insert(env.caller(), new_vec);
                },
                Some(a) => {
                    a.push(code_hash);
                }
            }
            env.emit(RegisterAbi {
                from: caller,
                name: name,
                code_hash: code_hash,
                abi: abi
            });
            true
        }

        /// Query abi by name
        pub(external) fn get_abi_by_name(&self, name: Hash) -> Text {
            self.name_to_abi.get(&name).unwrap_or(&Vec::new()).to_vec()
        }

        /// Query abi by code_hash
        pub(external) fn get_abi_by_code_hash(&self, code_hash: Hash) -> Text {
            self.code_hash_to_abi.get(&code_hash).unwrap_or(&Vec::new()).to_vec()
        }

        /// Query code_hash list by account
        pub(external) fn get_code_hash_list_by_account(&self, account: AccountId) -> Vec<Hash> {
            self.account_to_code_hash_list.get(&account).unwrap_or(&Vec::new()).to_vec()
        }

        /// Set address for specific name
        pub(external) fn set_address(&mut self, name: Hash, address: AccountId) -> bool {
            let caller: AccountId = env.caller();
            let owner: AccountId = self.get_owner_or_none(name);
            // env.println(&format!("set_address caller: {:?}, owner: {:?}", caller, owner));
            if caller != owner {
                return false
            }
            let old_address = self.name_to_address.insert(name, address);
            env.emit(SetAddress {
                name: name,
                from: caller,
                old_address: old_address,
                new_address: address,
            });
            return true
        }

        /// Transfer owner to another address
        pub(external) fn transfer(&mut self, name: Hash, to: AccountId) -> bool {
            let caller: AccountId = env.caller();
            let owner: AccountId = self.get_owner_or_none(name);
            // env.println(&format!("transfer caller: {:?}, owner: {:?}", caller, owner));
            if caller != owner {
                return false
            }
            let old_owner = self.name_to_owner.insert(name, to);
            env.emit(Transfer {
                name: name,
                from: caller,
                old_owner: old_owner,
                new_owner: to,
            });
            return true
        }

        /// Get address for the specific name 
        pub(external) fn get_address(&self, name: Hash) -> AccountId {
            let address: AccountId = self.get_address_or_none(name);
            // env.println(&format!("get_address name is {:?}, address is {:?}", name, address));
            address
        }

        /// Check whether name is exist
        pub(external) fn is_name_exist(&self, name: Hash) -> bool {
            self.is_name_exist_impl(name)
        }
    }

    /// Implement some private methods
    impl SimplePns {
        /// Returns an AccountId or default 0x00*32 if it is not set.
        fn get_address_or_none(&self, name: Hash) -> AccountId {
            let address = self.name_to_address.get(&name).unwrap_or(&self.default_address);
            *address
        }

        /// Returns an AccountId or default 0x00*32 if it is not set.
        fn get_owner_or_none(&self, name: Hash) -> AccountId {
            let owner = self.name_to_owner.get(&name).unwrap_or(&self.default_address);
            *owner
        }

        /// check whether name is exist
        fn is_name_exist_impl(&self, name: Hash) -> bool {
            let address = self.name_to_owner.get(&name);
            if let None = address {
                return false;
            }
            true
        }
    }

}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn register_works() {
        let alice = AccountId::from([0x1; 32]);
        // let bob: AccountId = AccountId::from([0x2; 32]);
        let name = Hash::from([0x99; 32]);

        let mut contract = SimplePns::deploy_mock();
        env::test::set_caller::<Types>(alice);

        assert_eq!(contract.register(name, alice), true);
        assert_eq!(contract.register(name, alice), false);
    }
    
    #[test]
    fn set_address_works() {
        let alice = AccountId::from([0x1; 32]);
        let bob: AccountId = AccountId::from([0x2; 32]);
        let name = Hash::from([0x99; 32]);

        let mut contract = SimplePns::deploy_mock();
        env::test::set_caller::<Types>(alice);

        assert_eq!(contract.register(name, alice), true);

        // caller is not owner, set_address will be failed
        env::test::set_caller::<Types>(bob);
        assert_eq!(contract.set_address(name, bob), false);

        // caller is owner, set_address will be successful
        env::test::set_caller::<Types>(alice);
        assert_eq!(contract.set_address(name, bob), true);

        assert_eq!(contract.get_address(name), bob);
    }

    #[test]
    fn transfer_works() {
        let alice = AccountId::from([0x1; 32]);
        let bob = AccountId::from([0x2; 32]);
        let name = Hash::from([0x99; 32]);

        let mut contract = SimplePns::deploy_mock();
        env::test::set_caller::<Types>(alice);

        assert_eq!(contract.register(name, alice), true);

        // transfer owner
        assert_eq!(contract.transfer(name, bob), true);

        // now owner is bob, alice set_address will be failed
        assert_eq!(contract.set_address(name, bob), false);

        env::test::set_caller::<Types>(bob);
        // now owner is bob, set_address will be successful
        assert_eq!(contract.set_address(name, bob), true);

        assert_eq!(contract.get_address(name), bob);
    }
}
