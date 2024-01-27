{
    override_text_style: None,
    override_font_id: None,
    text_styles: {
        Small: FontId { size: 9.0, family: Proportional },
        Body: FontId { size: 12.5, family: Proportional },
        Monospace: FontId { size: 12.0, family: Monospace },
        Button: FontId { size: 12.5, family: Proportional },
        Heading: FontId {size: 18.0, family: Proportional}
    }, 
    drag_value_text_style: Button, 
    wrap: None, 
    spacing: Spacing {
        item_spacing: [
            8.0 3.0
        ], window_margin: Margin {
            left: 6.0, right: 6.0, top: 6.0, bottom: 6.0
        }, button_padding: [
            4.0 1.0
        ], menu_margin: Margin {
            left: 6.0, right: 6.0, top: 6.0, bottom: 6.0
        }, indent: 18.0, interact_size: [
            40.0 18.0
        ], slider_width: 100.0, combo_width: 100.0, text_edit_width: 280.0, icon_width: 14.0, icon_width_inner: 8.0, icon_spacing: 4.0, tooltip_width: 600.0, indent_ends_with_horizontal_line: false, combo_height: 200.0, scroll: ScrollStyle {
            floating: true, bar_width: 12.0, handle_min_length: 12.0, bar_inner_margin: 4.0, bar_outer_margin: 0.0, floating_width: 2.0, floating_allocated_width: 0.0, foreground_color: true, dormant_background_opacity: 0.0, active_background_opacity: 0.4, interact_background_opacity: 0.7, dormant_handle_opacity: 0.0, active_handle_opacity: 0.6, interact_handle_opacity: 1.0
        }
    }, 
    interaction: Interaction {
        resize_grab_radius_side: 5.0, resize_grab_radius_corner: 10.0, show_tooltips_only_when_still: true, tooltip_delay: 0.0
    }, 
    visuals: Visuals {
        dark_mode: true, override_text_color: None, widgets: Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: Color32([
                    27,
                    27,
                    27,
                    255
                ]), 
                weak_bg_fill: Color32([
                    27,
                    27,
                    27,
                    255
                ]), 
                bg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        60,
                        60,
                        60,
                        255
                    ])
                }, 
                rounding: Rounding {
                    nw: 2.0, ne: 2.0, sw: 2.0, se: 2.0
                }, 
                fg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        140,
                        140,
                        140,
                        255
                    ])
                }, expansion: 0.0
            }, 
            inactive: WidgetVisuals {
                bg_fill: Color32([
                    60,
                    60,
                    60,
                    255
                ]), weak_bg_fill: Color32([
                    60,
                    60,
                    60,
                    255
                ]), 
                bg_stroke: Stroke {
                    width: 0.0, color: Color32([
                        0,
                        0,
                        0,
                        0
                    ])
                }, 
                rounding: Rounding {
                    nw: 2.0, ne: 2.0, sw: 2.0, se: 2.0
                }, 
                fg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        180,
                        180,
                        180,
                        255
                    ])
                }, expansion: 0.0
            }, 
            hovered: WidgetVisuals {
                bg_fill: Color32([
                    70,
                    70,
                    70,
                    255
                ]), weak_bg_fill: Color32([
                    70,
                    70,
                    70,
                    255
                ]), bg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        150,
                        150,
                        150,
                        255
                    ])
                }, rounding: Rounding {
                    nw: 3.0, ne: 3.0, sw: 3.0, se: 3.0
                }, fg_stroke: Stroke {
                    width: 1.5, color: Color32([
                        240,
                        240,
                        240,
                        255
                    ])
                }, expansion: 1.0
            }, active: WidgetVisuals {
                bg_fill: Color32([
                    55,
                    55,
                    55,
                    255
                ]), weak_bg_fill: Color32([
                    55,
                    55,
                    55,
                    255
                ]), bg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        255,
                        255,
                        255,
                        255
                    ])
                }, rounding: Rounding {
                    nw: 2.0, ne: 2.0, sw: 2.0, se: 2.0
                }, fg_stroke: Stroke {
                    width: 2.0, color: Color32([
                        255,
                        255,
                        255,
                        255
                    ])
                }, expansion: 1.0
            }, open: WidgetVisuals {
                bg_fill: Color32([
                    27,
                    27,
                    27,
                    255
                ]), weak_bg_fill: Color32([
                    45,
                    45,
                    45,
                    255
                ]), bg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        60,
                        60,
                        60,
                        255
                    ])
                }, rounding: Rounding {
                    nw: 2.0, ne: 2.0, sw: 2.0, se: 2.0
                }, fg_stroke: Stroke {
                    width: 1.0, color: Color32([
                        210,
                        210,
                        210,
                        255
                    ])
                }, expansion: 0.0
            }
        }, selection: Selection {
            bg_fill: Color32([
                0,
                92,
                128,
                255
            ]), stroke: Stroke {
                width: 1.0, color: Color32([
                    192,
                    222,
                    255,
                    255
                ])
            }
        }, hyperlink_color: Color32([
            90,
            170,
            255,
            255
        ]), faint_bg_color: Color32([
            5,
            5,
            5,
            0
        ]), extreme_bg_color: Color32([
            10,
            10,
            10,
            255
        ]), code_bg_color: Color32([
            64,
            64,
            64,
            255
        ]), warn_fg_color: Color32([
            255,
            143,
            0,
            255
        ]), error_fg_color: Color32([
            255,
            0,
            0,
            255
        ]), window_rounding: Rounding {
            nw: 6.0, ne: 6.0, sw: 6.0, se: 6.0
        }, window_shadow: Shadow {
            extrusion: 32.0, color: Color32([
                0,
                0,
                0,
                96
            ])
        }, window_fill: Color32([
            27,
            27,
            27,
            255
        ]), window_stroke: Stroke {
            width: 1.0, color: Color32([
                60,
                60,
                60,
                255
            ])
        }, window_highlight_topmost: true, menu_rounding: Rounding {
            nw: 6.0, ne: 6.0, sw: 6.0, se: 6.0
        }, panel_fill: Color32([
            27,
            27,
            27,
            255
        ]), popup_shadow: Shadow {
            extrusion: 16.0, color: Color32([
                0,
                0,
                0,
                96
            ])
        }, resize_corner_size: 12.0, text_cursor: Stroke {
            width: 2.0, color: Color32([
                192,
                222,
                255,
                255
            ])
        }, text_cursor_preview: false, clip_rect_margin: 3.0, button_frame: true, collapsing_header_frame: false, indent_has_left_vline: true, striped: false, slider_trailing_fill: false, handle_shape: Circle, interact_cursor: None, image_loading_spinners: true, numeric_color_space: GammaByte
    }, animation_time: 0.083333336, debug: DebugOptions {
        debug_on_hover: false, debug_on_hover_with_all_modifiers: true, hover_shows_next: false, show_expand_width: false, show_expand_height: false, show_resize: false, show_interactive_widgets: false, show_blocking_widget: false
    }, explanation_tooltips: false, always_scroll_the_only_direction: false
}