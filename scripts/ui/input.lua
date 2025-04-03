-- Register a keyboard shortcut
ui.input.register_shortcut("SHIFT+X", function()
	print("SHIFT+X was pressed!")
end)

ui.input.register_shortcut("R", function()
	print("x was pressed!")
end)
-- Register a mouse click handler
ui.input.register_mouse("LMB", function()
	local x, y = ui.tile.hovered()
	local person = api.person.create("testa", x, y)
	print_compact("Person:", person)
end)
