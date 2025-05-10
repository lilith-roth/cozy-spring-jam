@tool
extends BTAction

@export var target_var: StringName = &"target"


func _generate_name() -> String:
	return "Flanking target at: %s" % [
		LimboUtility.decorate_var(target_var
	)]
	
# Called each time this task is entered.
func _enter() -> void:
	pass
	
# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	var target: Node2D = blackboard.get_var(target_var, null)
	if not is_instance_valid(target):
		return FAILURE
	var direction = Vector2.RIGHT.rotated(round(target.global_position.angle() / TAU * 8) * TAU / 8).snapped(Vector2.ONE)
	print(direction)
	#var target_pos: Vector2 = 
	return SUCCESS
