@tool
extends BTAction

@export
var target_var: StringName = &"target"

@export
var path_desired_distance: float = 4.0

@export
var target_desired_distance: float = 4.0

var nav_agent: NavigationAgent2D

func _generate_name() -> String:
	return "Going to: %s" % [LimboUtility.decorate_var(target_var)]
	
# Called each time this task is entered.
func _enter() -> void:
	nav_agent = agent.find_child("NavigationAgent2D")
	var target: Node2D = blackboard.get_var(target_var, null)
	if is_instance_valid(target):
		actor_setup.call_deferred(target)
	else:
		assert(false, "No target set before moving!!!")
	

func _exit() -> void:
	nav_agent.set_target_position(Vector2.ZERO)

# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	#if nav_agent.target_position == Vector2.ZERO:
		#return FAILURE
	if nav_agent.is_navigation_finished():
		return SUCCESS
	agent.move_towards_target()
	return RUNNING

func actor_setup(target: Node2D):
	# Wait for the first physics frame so the NavigationServer can sync.
	nav_agent.set_target_position(target.global_position)
	print("Set target to " + str(target.global_position))
