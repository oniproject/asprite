
enum Split {
	V(Container, Container),
	H(Container, Container),
	Tabs(usize, Vec<Container>),
	None,
}

enum Slot {
	Tab,

	Left,
	Right,
	Top,
	Bottom,
}

fn dock(slot: Slot, c: Container) {
	match slot {
	Slot::Left => self.split = Split::V(self.split, c),
	Slot::Right => self.split = Split::V(c, self.split),

	Slot::Top => self.split = Split::H(c, self.split),
	Slot::Bottom => self.split = Split::H(self.split, c),
	}
}

/*
struct Container {
	split: (Option<Container>, Option<Container>),
	parent: *mut Container,
	active_dock: *mut Dock, // = nullptr;

	docs: Vec<*mut Dock>,

	vertical: bool, //= false;
	always_auto_resize: bool, //= true;
	size: f32, //= 0
}

struct Dock {
	Dock *initialize(const char *dtitle,  bool dcloseButton, ImVec2 dminSize, std::function<void(ImVec2)> ddrawFunction) {
		title = dtitle;
		closeButton = dcloseButton;
		minSize = dminSize;
		drawFunction = ddrawFunction;
		return this;
	};

	//Container *parent = nullptr;
	Container *container = nullptr;
	Dockspace *redockFrom = nullptr;
	Dock *redockTo = nullptr;

	const char *title;
	DockSlot dockSlot = DockSlot::Tab;
	DockSlot redockSlot = DockSlot::None;
	bool closeButton = true;
	bool undockable = false;
	bool draging = false;
	ImVec2 lastSize;
	ImVec2 minSize;

	std::function<void(ImVec2)> drawFunction;
	std::function<bool(void)> onCloseFunction;
}

enum DockToAction {
	Undock, Drag, Close, Null
}
	
class Dockspace {
public:
	Dockspace();
	~Dockspace();

	bool dock(Dock *dock, DockSlot dockSlot, float size = 0, bool active = false);
	bool dockWith(Dock *dock, Dock *dockTo, DockSlot dockSlot, float size = 0, bool active = false);
	bool undock(Dock *dock);
	
	void updateAndDraw(ImVec2 size);
	void clear();

	std::vector<Dock*>m_docks;

	Container m_container;
	std::vector<Container*>m_containers;

protected:

	void _renderTabBar(Container *container, ImVec2 size, ImVec2 cursorPos);
	bool _getMinSize(Container *container, ImVec2 *min);


	Dock *m_currentDockTo = nullptr;
	DockToAction m_currentDockToAction = eNull;
};


	fn dock(&mut self, dock: &mut Dock, to: Option<&mut Dock>, slot: Option<Slot>, size: f32, active: bool) -> bool {
		let current = &self.container;

		if let Some(to) = to {
			if slot == Slot::Tab {
				to.container.active_dock = if active || !current.splits[0].active_dock {
					dock
				} else {
					 current.splits[0].activeDock
				};
				to.container.docks.push_back(dock);
				dock.container = dockTo.container;
				return true;
			} else {
				self.containers.push_back(new Container{});
				let new = self.containers[self.containers.size() - 1];
				new.parent = to.container.parent;
				new.splits[0] = to.container;
				new.size = to.container.size;

				//if (size)
				//	newContainer.alwaysAutoResize = false;

				to.container.size = 0;

				if to.container.parent.splits[0] == to.container {
					to.container.parent.splits[0] = new;
				} else {
					to.container.parent.splits[1] = new;
				}
				//to.container.parent = new;
				to.container = new.splits[0];
				to.container.parent = new;
				current = new;
			}
		}

		let child = if current.splits[0].is_none() || current.splits[1].is_none() {
			self.containers.push_back(new Container{});
			Some(self.containers[m_containers.size() - 1])
		} else {
			None
		}

		if current.splits.0.is_none() {
			current.splits.0 = child;
			current.splits.0.active_dock = active ? dock : currentContainer.splits[0].activeDock ? currentContainer.splits[0].activeDock : dock;
			current.splits.0.docks.push_back(dock);
			current.splits.0.parent = current;
			current.splits.0.size = size.abs();
			dock.container = current.splits[0];
			dock.container.parent = current;
		} else if (currentContainer.splits[1] == nullptr) {
			current.splits[1] = childContainer;
			let otherSplit = currentContainer.splits.0;
			if size > 0 {
				current.splits.0.alwaysAutoResize = true;
				current.splits.0.size = 0;
				current.splits.1.size = size;
				current.splits.1.alwaysAutoResize = false;
			} else if size == 0 {
				// pass
			} else {
				current.splits.0.alwaysAutoResize = false;
				current.splits.0.size = size * -1;
				current.splits.1.size = 0;
				current.splits.1.alwaysAutoResize = true;
			}
			if let Some(slot) = slot {
				match slot {
				Slot::Left => {
					current.splits[1] = current.splits[0];
					current.splits[0] = child;
					current.vertical = true;
				}
				Slot::Top => {
					current.splits[1] = current.splits[0];
					current.splits[0] = child;
					current.vertical = false;
				}
				Slot::Right => current.vertical = true,
				Slot::Bottom => current.vertical = false,
				Slot::Tab => current.vertical = false,
				}
			}
			child.active_dock = active ? dock : child.active_dock ? childContainer.activeDock : dock;
			child.docks.push_back(dock);
			child.parent = current;

			//	if (childContainer.parent != nullptr && currentContainer.verticalSplit != childContainer.parent.verticalSplit)
			//		currentContainer.size = otherSplit.size ? otherSplit.size + otherSplit.size : otherSplit.size;

			dock.container = child;
		} else {
			return false;
		}
		true
	}

	fn undock(Dock *dock) -> bool {
		if (dock != nullptr)
		{
			if (dock.container.docks.size() > 1)
			{
				for (int i = 0; i < dock.container.docks.size(); i++)
				{
					if (dock.container.docks[i] == dock)
					{
						dock.lastSize = dock.container.activeDock.lastSize;
						dock.container.docks.erase(dock.container.docks.begin() + i);
						if (i != dock.container.docks.size())
							dock.container.activeDock = dock.container.docks[i];
						else dock.container.activeDock = dock.container.docks[i - 1];
					}
				}
			}
			else
			{
				Container *toDelete = nullptr, *parentToDelete = nullptr;
				if (dock.container.parent == &m_container)
				{
					if (m_container.splits[0] == dock.container)
					{
						if (m_container.splits[1])
						{
							toDelete = m_container.splits[0];
							if(m_container.splits[1].splits[0]){
								parentToDelete = m_container.splits[1];
								self.container.splits.0 = self.container.splits[1].splits[0];
								self.container.splits.0.parent = &m_container;
								self.container.splits.0.verticalSplit = false;
								self.container.splits.1 = self.container.splits[1].splits[1];
								self.container.splits.1.parent = &m_container;
								self.container.splits.1.parent.vertical = self.container.splits[1].verticalSplit;
								self.container.splits.1.verticalSplit = false;
							}
							else 
							{ 
								m_container.splits[0] = m_container.splits[1]; 
								m_container.splits[1] = nullptr;
								m_container.splits[0].size = 0;
								m_container.splits[0].verticalSplit = false;
								m_container.splits[0].parent.verticalSplit = false;
							}
						} else {
							return false;
						}
					}
					else
					{
						toDelete = m_container.splits[1];
						m_container.splits[1] = nullptr;
					}
				}
				else
				{
					parentToDelete = dock.container.parent;
					if (dock.container.parent.splits[0] == dock.container)
					{
						toDelete = dock.container.parent.splits[0];
						Container *parent = dock.container.parent.parent;
						Container *working = nullptr;
						if (dock.container.parent.parent.splits[0] == dock.container.parent)
							working = dock.container.parent.parent.splits[0] = dock.container.parent.splits[1];
						else working = dock.container.parent.parent.splits[1] = dock.container.parent.splits[1];
						working.parent = parent;
						working.size =  dock.container.parent.size;
					}
					else
					{
						toDelete = dock.container.parent.splits[1];
						Container *parent = dock.container.parent.parent;
						Container *working = nullptr;
						if (dock.container.parent.parent.splits[0] == dock.container.parent)
							working = dock.container.parent.parent.splits[0] = dock.container.parent.splits[0];
						else working = dock.container.parent.parent.splits[1] = dock.container.parent.splits[0];
						working.parent = parent;
						working.size = dock.container.parent.size;
					}
				}
				for (int i = 0; i < m_containers.size(); i++)
				{
					if (toDelete == m_containers[i])
					{
						delete m_containers[i];
						m_containers.erase(m_containers.begin() + i);
					}
					if(m_containers.size() > 1 && parentToDelete == m_containers[i])
					{
						delete m_containers[i];
						m_containers.erase(m_containers.begin() + i);
					}
					if (m_containers.size() > 1 && toDelete == m_containers[i])
					{
						delete m_containers[i];
						m_containers.erase(m_containers.begin() + i);
					}
				}
			}
			return true;
		}
		return false;
	}

	void Dockspace::updateAndDraw(ImVec2 dockspaceSize)
	{
		uint32_t idgen = 0;

		float tabbarHeight = 20;

		std::function<void(Container*, ImVec2, ImVec2)> renderContainer = [&](Container *container, ImVec2 size, ImVec2 cursorPos) {
			ImVec2 calculatedSize = size;
			ImVec2 calculatedCursorPos = cursorPos;

			idgen++;

			std::string idname = "Dock##";
			idname += idgen;

			calculatedSize.y -= tabbarHeight;

			float splitterButtonWidth = 4;
			float splitterButtonWidthHalf = splitterButtonWidth / 2;

			if (container.splits[0] == nullptr && container != &m_container)
			{
				_renderTabBar(container, calculatedSize, cursorPos);
				cursorPos.y += tabbarHeight;
				
				ImGui::SetCursorPos(cursorPos);
				ImVec2 screenCursorPos = ImGui::GetCursorScreenPos();
				screenCursorPos.y -= tabbarHeight;

				ImGui::PushStyleColor(ImGuiCol_ChildWindowBg, ImVec4(.25, .25, .25, 1));
				ImGui::BeginChild(idname.c_str(), calculatedSize, false, ImGuiWindowFlags_AlwaysUseWindowPadding);
				container.activeDock.drawFunction(calculatedSize);
				container.activeDock.lastSize = calculatedSize;

				ImGui::EndChild();
				ImGui::PopStyleColor(1);
			}
			else
			{
				ImVec2 calculatedSize0 = size, calculatedSize1;

				if (container.splits[1])
				{
					float acontsizeX = container.splits[0].size ? container.splits[0].size :
						container.splits[1].size ? size.x - container.splits[1].size - splitterButtonWidth : size.x / 2 - splitterButtonWidthHalf;
					float acontsizeY = container.splits[0].size ? container.splits[0].size :
						container.splits[1].size ? size.y - container.splits[1].size - splitterButtonWidth : size.y / 2 - splitterButtonWidthHalf;

					float bcontsizeX = container.splits[0].size ? size.x - container.splits[0].size - splitterButtonWidth :
						container.splits[1].size ? container.splits[1].size : size.x / 2 - splitterButtonWidthHalf;
					float bcontsizeY = container.splits[0].size ? size.y - container.splits[0].size - splitterButtonWidth :
						container.splits[1].size ? container.splits[1].size : size.y / 2 - splitterButtonWidthHalf;

					calculatedSize0 = ImVec2(container.verticalSplit ? acontsizeX : size.x, !container.verticalSplit ? acontsizeY : size.y);
					calculatedSize1 = ImVec2(container.verticalSplit ? bcontsizeX : size.x, !container.verticalSplit ? bcontsizeY : size.y);
				}
				if (container.splits[0])
				{
					if (container.splits[0] == nullptr)
						size.x = 1;
					renderContainer(container.splits[0], calculatedSize0, calculatedCursorPos);
					if (container.verticalSplit)
						calculatedCursorPos.x = calculatedSize0.x + calculatedCursorPos.x + splitterButtonWidth;
					else
					{
						calculatedCursorPos.y = calculatedSize0.y + calculatedCursorPos.y + splitterButtonWidth;
					}
				}
				Container *thisContainer = container.splits[1];
				if (container.splits[1])
				{
					ImGui::SetCursorPosX(calculatedCursorPos.x - splitterButtonWidth);
					ImGui::SetCursorPosY(calculatedCursorPos.y - splitterButtonWidth);
					std::string idnamesb = "##SplitterButton";
					idnamesb += idgen++;
					ImGui::InvisibleButton(idnamesb.c_str(), ImVec2(
						container.verticalSplit ? splitterButtonWidth : size.x + splitterButtonWidth,
						!container.verticalSplit ? splitterButtonWidth : size.y + splitterButtonWidth));

					ImGui::SetItemAllowOverlap(); // This is to allow having other buttons OVER our splitter. 

					if (ImGui::IsItemActive())
					{
						float mouse_delta = !container.verticalSplit ? ImGui::GetIO().MouseDelta.y : ImGui::GetIO().MouseDelta.x;

						if (container.splits[0].alwaysAutoResize != true)
						{
							ImVec2 minSize;
							_getMinSize(container.splits[0], &minSize);
							if (container.splits[0].size == 0)
								container.splits[0].size = container.verticalSplit ? calculatedSize1.x : calculatedSize1.y;
							if (container.splits[0].size + mouse_delta >= (container.verticalSplit ? minSize.x : minSize.y))
								container.splits[0].size += mouse_delta;
						}
						else
						{
							ImVec2 minSize;
							_getMinSize(container.splits[1], &minSize);
							if (container.splits[1].size == 0)
								container.splits[1].size = container.verticalSplit ? calculatedSize1.x : calculatedSize1.y;
							if (container.splits[1].size - mouse_delta >= (container.verticalSplit ? minSize.x : minSize.y))
								container.splits[1].size -= mouse_delta;
						}
					}

					if (ImGui::IsItemHovered() || ImGui::IsItemActive())
						//ImGui::SetMouseCursor(container.verticalSplit ? ImGuiMouseCursor_ResizeNS : ImGuiMouseCursor_ResizeEW);
						SetCursor(LoadCursor(NULL, container.verticalSplit ? IDC_SIZEWE : IDC_SIZENS));

					renderContainer(container.splits[1], calculatedSize1, calculatedCursorPos);
				}
			}
		};

		ImVec2 backup_pos = ImGui::GetCursorPos();
		renderContainer(&m_container, dockspaceSize, backup_pos);
		ImGui::SetCursorPos(backup_pos);
	};

	void Dockspace::clear()
	{
		for (auto container : m_containers)
		{
			delete container;
		}
		m_containers.clear();

		m_container = {};
	};

	fn _getMinSize(Container *container, ImVec2 *min) -> bool {
		int begin = 0;
		if (ontainer.splits[0] == nullptr {
			min.x = min.x.min(container.activeDock.minSize.x);
			min.y = min.y.min(container.activeDock.minSize.y);
			true
		} else {
			_getMinSize(container.splits[0], min) && container.splits[1] && _getMinSize(container.splits[1], min)
		}
	};

	void Dockspace::_renderTabBar(Container *container, ImVec2 size, ImVec2 cursorPos)
	{
		ImGui::SetCursorPos(cursorPos);

		ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2(14, 3));
		for (auto dock : container.docks)
		{
			std::string dockTitle = dock.title;
			if (dock.closeButton == true)
				dockTitle += "  ";

			if (dock == container.activeDock)
			{
				ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(.25, .25, .25, 1));
				ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4(.25, .25, .25, 1));
				ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4(.25, .25, .25, 1));
			}
			else
			{
				ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(.21, .21, .21, 1));
				ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4(.35, .35, .35, 1));
				ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4(.4, .4, .4, 1));
			}
			if (ImGui::Button(dockTitle.c_str(), ImVec2(0, 20)))
			{
				container.activeDock = dock;
			}

			ImGui::SameLine();
			ImGui::PopStyleColor(3);
		}
		ImGui::PopStyleVar();
	};
}
*/