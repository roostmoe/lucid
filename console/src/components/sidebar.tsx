import { useAuth } from "@/lib/state/auth";
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "./ui/sidebar";
import { Collapsible } from "@/components/ui/collapsible";
import { IconDashboard } from "@tabler/icons-react";

export const AppSidebar = () => {
  return (
    <Sidebar variant="inset">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" render={(
              <a href="#">
                <div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
                  L
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-medium">Lucid</span>
                </div>
              </a>
            )} />
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        <AppSidebarNavMain />
      </SidebarContent>

      <SidebarFooter>
        <AppSidebarNavUser />
      </SidebarFooter>
    </Sidebar>
  );
};

export const AppSidebarNavMain = () => {
  return (
    <SidebarGroup>
      <SidebarGroupLabel>Home</SidebarGroupLabel>
      <SidebarMenu>
        <Collapsible defaultOpen render={(
          <SidebarMenuItem>
            <SidebarMenuButton render={(
              <a href="#">
                <IconDashboard />
                <span>Dashboard</span>
              </a>
            )} />
          </SidebarMenuItem>
        )}/>
      </SidebarMenu>
    </SidebarGroup>
  );
};

export const AppSidebarNavUser = () => {
  const { user, logout } = useAuth();

  return (
    <span onClick={() => logout()}>
      Hello, {user?.display_name}.
    </span>
  );
};
