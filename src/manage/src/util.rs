use ic_cdk::export::candid::Principal;

pub fn is_controller(content: String, controller: Principal) -> bool {
    let content: Vec<&str> = content.split("\n").collect();
    let controller = controller.to_string();
    let controller_content: Vec<&str> = content[1].split(" ").collect();
    controller_content.contains(&controller.as_str())
}
