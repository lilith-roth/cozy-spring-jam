@tool
extends BTAction

@export var target_var: StringName = &"target"

@export var range_min: int = 300
@export var range_max: int = 400


func _generate_name() -> String:
	return "Check in range [%s, %s] to: %s" % [
		range_min,
		range_max,
		LimboUtility.decorate_var(target_var
	)]
	
# Called each time this task is entered.
func _enter() -> void:
	pass
	
# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	var target: Node2D = blackboard.get_var(target_var, null)
	var current_pos: Vector2 = agent.global_position
	var dist = current_pos.distance_squared_to(target.global_position)
	if dist > range_min && dist < range_max:
		return SUCCESS
	return FAILURE
