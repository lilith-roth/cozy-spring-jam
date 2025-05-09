extends Node


var basic_melee_enemy: PackedScene

func _ready() -> void:
	print("Loading console")
	basic_melee_enemy = preload("res://scenes/npcs/enemies/basic_melee_enemy.tscn")
	LimboConsole.register_command(spawn_enemy)


func spawn_enemy(position_x: int, position_y: int) -> void:
	var enemy_instance: RigidBody2D = basic_melee_enemy.instantiate()
	enemy_instance.position = Vector2()
	var npc_node = get_tree().get_current_scene().get_node("/root/npcs")
	if npc_node == null:
		var npc_root: Node2D = Node2D.new()
		npc_root.name = "npcs"
		get_tree().get_current_scene().get_node("/root").add_child(npc_root)
		npc_node = get_tree().get_current_scene().get_node("/root/npcs")
	npc_node.add_child(enemy_instance)
	enemy_instance.global_position = Vector2(position_x, position_y)
