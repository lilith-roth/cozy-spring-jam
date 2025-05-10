@tool
extends BTAction

@export var target_var: StringName = &"target"

@export var distance_min: float = 100
@export var distance_max: float = 150


var nav_agent: NavigationAgent2D


func _generate_name() -> String:
	return "Flanking target at: %s" % [
		LimboUtility.decorate_var(target_var
	)]
	
	
# Called each time this task is entered.
func _enter() -> void:
	nav_agent = agent.find_child("NavigationAgent2D")
	var target: Node2D = blackboard.get_var(target_var, null)
	if is_instance_valid(target):
		actor_setup.call_deferred(target)
	else:
		assert(false, "No target set before moving!!!")
	
# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	if nav_agent.is_navigation_finished():
		return SUCCESS
	agent.move_towards_target()
	return RUNNING

func actor_setup(target: Node2D):
	# Wait for the first physics frame so the NavigationServer can sync.
	var direction = agent.global_position.direction_to(target.global_position)
	var position_offset = Vector2(
		agent.global_position.x + (direction.x * randf_range(distance_min, distance_max)), 
		agent.global_position.y + (direction.y * randf_range(distance_min, distance_max))
	)
	nav_agent.set_target_position(position_offset)
	
