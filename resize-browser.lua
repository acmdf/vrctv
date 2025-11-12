obs = obslua

scale = 2

function script_properties()
        local props = obs.obs_properties_create()

        -- Scale factor
        local p = obs.obs_properties_add_float(props, "scale", "Scale Factor", 0, 10, 0.5)

        return props
end

function script_update(settings)
        scale = obs.obs_data_get_double(settings, "scale")
end

function script_load(settings)
        stuff = {}
        local scenes = obs.obs_frontend_get_scenes()
        for _, scene in pairs(scenes) do
                local name = obs.obs_source_get_name(scene)
                local sh = obs.obs_source_get_signal_handler(scene)
                obs.signal_handler_connect(sh, "item_transform", item_transform)
                
        end
        obs.source_list_release(scenes)

        local sh = obs.obs_get_signal_handler()
        obs.signal_handler_connect(sh, "source_create", source_create)
end

function source_create(cd)
        local source = obs.calldata_source(cd, "source")
        if obs.obs_source_is_scene(source) then
                local name = obs.obs_source_get_name(source)
                local sh = obs.obs_source_get_signal_handler(source)
                obs.signal_handler_connect(sh, "item_transform", item_transform)
        end
end

function fit_to_bounds(item, skip_cache)
        source = obs.obs_sceneitem_get_source(item)
        local source_type = obs.obs_source_get_unversioned_id(source)

        if source_type ~= "browser_source" then
                return
        end
        local data = obs.obs_source_get_settings(source)
        info = obs.obs_transform_info()
        obs.obs_sceneitem_get_info2(item, info)
        local width = info.bounds.x
        local height = info.bounds.y
        if width ~= 0 and height ~= 0 then
                obs.obs_data_set_int(data, "width", width*scale)
                obs.obs_data_set_int(data, "height", height*scale)
                obs.obs_source_update(source, data)
        end
        obs.obs_data_release(data)
end

function item_transform(cd)
        local item = obs.calldata_sceneitem(cd, "item")
        if item ~= nil then
                fit_to_bounds(item, false)
	end
end