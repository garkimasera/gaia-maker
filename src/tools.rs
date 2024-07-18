macro_rules! define_asset_list_from_enum {
    (
        #[asset(dir_path = $dir_path:expr)]
        #[asset(extension = $file_ext:expr)]
        $struct_vis:vis struct $struct_name:ident {
            $field_vis:vis $field:ident: HashMap<$enum_name:ident, Handle<$asset_type:ty>>,
        }
    ) => {
        #[derive(bevy::prelude::Resource)]
        $struct_vis struct $struct_name {
            $field_vis $field: HashMap<$enum_name, Handle<$asset_type>>,
        }

        impl $struct_name {
            pub fn get(&self, e: $enum_name) -> Handle<$asset_type> {
                self.$field[&e].clone()
            }
        }

        impl AssetCollection for $struct_name {
            fn create(world: &mut World) -> Self {
                world.resource_scope(
                    |world, _asset_keys: Mut<::bevy_asset_loader::dynamic_asset::DynamicAssets>| {
                        let mut map = HashMap::default();
                        for e in <$enum_name as strum::IntoEnumIterator>::iter() {
                            let s: &str = e.as_ref();
                            let asset_server = world
                                .get_resource::<AssetServer>()
                                .expect("Cannot get AssetServer");
                            let handle = asset_server
                                .get_handle(&format!("{}/{}.{}", $dir_path, s, $file_ext)).unwrap();
                            map.insert(e, handle);
                        }
                        $struct_name { $field: map }
                    },
                )
            }

            fn load(world: &mut World) -> Vec<UntypedHandle> {
                let asset_server = world
                    .get_resource::<AssetServer>()
                    .expect("Cannot get AssetServer");
                let _asset_keys = world
                    .get_resource::<bevy_asset_loader::prelude::DynamicAssets>()
                    .expect("Cannot get bevy_asset_loader::prelude::DynamicAssets");

                <$enum_name as strum::IntoEnumIterator>::iter()
                    .map(|e| {
                        let s: &str = e.as_ref();
                        asset_server.load_untyped(&format!("{}/{}.{}", $dir_path, s, $file_ext)).untyped()
                    })
                    .collect()
            }
        }
    };
}
