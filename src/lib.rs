#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::storage;
use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod simple_pns {
    #[ink(storage)]
    struct SimplePns {
        /// A hashmap to store all name to addresses mapping.
        name_to_address: storage::HashMap<Hash, AccountId>,
        /// A hashmap to store all name to owners mapping.
        name_to_owner: storage::HashMap<Hash, AccountId>,
    }

    /// Emitted whenever a new name is being registered.
    #[ink(event)]
    struct Register {
        #[ink(topic)]
        name: Hash,
        #[ink(topic)]
        from: AccountId,
    }

    /// Emitted whenever an address changes.
    #[ink(event)]
    struct SetAddress {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_address: Option<AccountId>,
        #[ink(topic)]
        new_address: AccountId,
    }

    /// Emitted whenver a name is being transferred.
    #[ink(event)]
    struct Transfer {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_owner: Option<AccountId>,
        #[ink(topic)]
        new_owner: AccountId,
    }

    impl SimplePns {
        /// Creates a new domain name service contract.
        #[ink(constructor)]
        fn new(&mut self) {

        }

        /// Register specific name with caller as owner.
        #[ink(message)]
        fn register(&mut self, name: Hash) -> bool {
            let caller = self.env().caller();
            if self.name_exists(name) {
                return false
            }
            self.name_to_owner.insert(name, caller);
            self.env().emit_event(Register { name, from: caller });
            true
        }

        /// Set address for specific name.
        #[ink(message)]
        fn set_address(&mut self, name: Hash, new_address: AccountId) -> bool {
            let caller = self.env().caller();
            let owner = self.name_to_owner.get(&name).cloned();
            if owner == None {
                return false
            }
            if Some(caller) != owner {
                return false
            }
            let old_address = self.name_to_address.insert(name, new_address);
            self.env().emit_event(SetAddress {
                name,
                from: caller,
                old_address,
                new_address,
            });
            true
        }

        /// Transfer owner to another address.
        #[ink(message)]
        fn transfer(&mut self, name: Hash, to: AccountId) -> bool {
            let caller = self.env().caller();
            let owner = self.name_to_owner.get(&name).cloned();
            if owner == None {
                return false
            }
            if Some(caller) != owner {
                return false
            }
            let old_owner = self.name_to_owner.insert(name, to);
            self.env().emit_event(Transfer {
                name,
                from: caller,
                old_owner,
                new_owner: to,
            });
            true
        }

        /// Get address for specific name.
        #[ink(message)]
        fn get_address(&self, name: Hash) -> Option<AccountId> {
            self.name_to_address.get(&name).cloned()
        }

        /// Returns `true` if the name already exists.
        #[ink(message)]
        fn is_name_exist(&self, name: Hash) -> bool {
            self.name_exists(name)
        }

        fn name_exists(&self, name: Hash) -> bool {
            if self.name_to_owner.get(&name).is_some() {
                return true
            }
            false
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
