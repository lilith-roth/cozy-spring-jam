@tool
extends BTAction

@export var target_var: StringName = &"target"


func _generate_name() -> String:
	return "Shooting at: %s" % [
		LimboUtility.decorate_var(target_var
	)]
	
# Called each time this task is entered.
func _enter() -> void:
	pass
	
# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	var target: Node2D = blackboard.get_var(target_var, null)
	agent.shoot(target.global_position)
	return SUCCESS
