import { useAuth } from "@/lib/state/auth";
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "./ui/sidebar";
import { IconChecklist, IconDashboard, IconLogout, IconServer, IconTriangle, type Icon } from "@tabler/icons-react";
import { Link, useRouterState } from "@tanstack/react-router";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { useIsMobile } from "@/hooks/use-mobile";

type AppSidebarItem = {
  type: 'group';
  title: string;
  items: AppSidebarItem[];
} | {
  type: 'item';
  title: string;
  url: string;
  activeExact?: boolean;
  icon?: Icon;
} | {
  type: 'collapsible';
  title: string;
  url: string;
  icon?: Icon;
  items: AppSidebarItem[];
};

const sidebarItems = [
  {
    type: 'group',
    title: '',
    items: [
      {
        type: 'item',
        title: 'Dashboard',
        url: '/',
        activeExact: true,
        icon: IconDashboard,
      },
    ],
  },
  {
    type: 'group',
    title: 'Inventory',
    items: [
      {
        type: 'item',
        title: 'Hosts',
        url: '/#hosts',
        icon: IconServer,
      },
    ],
  },
  {
    type: 'group',
    title: 'Security',
    items: [
      {
        type: 'item',
        title: 'Vulnerabilities',
        url: '/#vulns',
        icon: IconTriangle,
      },
      {
        type: 'item',
        title: 'Compliance',
        url: '/#compliance',
        icon: IconChecklist,
      },
    ],
  },
] as AppSidebarItem[];

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
  return sidebarItems.map((v, idx) => <AppSidebarNavItem key={idx} item={v} />);
};

export const AppSidebarNavItem = ({ item }: { item: AppSidebarItem }) => {
  const { location } = useRouterState();

  switch (item.type) {
    case 'group':
      return (
        <SidebarGroup>
          {item.title != "" && (
            <SidebarGroupLabel>{item.title}</SidebarGroupLabel>
          )}

          <SidebarMenu>
            {item.items.map((item, idx) => <AppSidebarNavItem key={idx} item={item} />)}
          </SidebarMenu>
        </SidebarGroup>
      );

    case 'item':
      const isActive = (
        item.activeExact
          ? location.pathname === item.url
          : location.pathname.startsWith(item.url)
      );

      return (
        <SidebarMenuItem>
          <SidebarMenuButton isActive={isActive} render={(
            <Link to={item.url}>
              {item.icon && <item.icon />}
              <span>{item.title}</span>
            </Link>
          )} />
        </SidebarMenuItem>
      );
  }
};

export const AppSidebarNavUser = () => {
  const { user, logout } = useAuth();
  const isMobile = useIsMobile();

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger render={(
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">{user?.display_name}</span>
                <span className="truncate font-xs text-muted-foreground">{user?.email}</span>
              </div>
            </SidebarMenuButton>
          )} />

          <DropdownMenuContent
            className="w-(--base-ui-dropdown-menu-trigger-width) min-w-56 rounded-lg"
            side={isMobile ? "bottom" : "right"}
            align="end"
            sideOffset={4}
          >
            <DropdownMenuGroup>
              <DropdownMenuLabel className="p-0 font-normal">
                <div className="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                  <div className="grid flex-1 text-left text-sm leading-tight">
                    <span className="truncate font-medium">{user?.display_name}</span>
                    <span className="truncate text-xs">{user?.email}</span>
                  </div>
                </div>
              </DropdownMenuLabel>
            </DropdownMenuGroup>

            <DropdownMenuSeparator />

            <DropdownMenuGroup>
              <DropdownMenuItem onClick={() => logout()}>
                <IconLogout />
                Log out
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
};
