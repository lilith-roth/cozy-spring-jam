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
	actor_setup.call_deferred()
	

# Called each time this task is ticked (aka executed).
func _tick(_delta: float) -> Status:
	if nav_agent.is_navigation_finished():
		return SUCCESS
	agent.move_towards_target()
	return RUNNING

func actor_setup():
	# Wait for the first physics frame so the NavigationServer can sync.
	await agent.get_tree().physics_frame
