import { useAuth } from "@/lib/state/auth";
import { Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarTrigger } from "./ui/sidebar";
import { IconBox, IconChecklist, IconDashboard, IconDatabase, IconExclamationCircle, IconKey, IconLogout, IconServer, IconTriangle, type Icon } from "@tabler/icons-react";
import { Link, useRouterState } from "@tanstack/react-router";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { useIsMobile } from "@/hooks/use-mobile";
import type { PropsWithChildren } from "react";
import { Badge } from "./ui/badge";

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
  badge?: number;
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
        url: '/hosts',
        icon: IconServer,
      },
      {
        type: 'item',
        title: 'Activation Keys',
        url: '/#activation-keys',
        icon: IconKey,
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
        badge: 6,
      },
      {
        type: 'item',
        title: 'Compliance',
        url: '/#compliance',
        icon: IconChecklist,
      },
    ],
  },
  {
    type: 'group',
    title: 'Content',
    items: [
      {
        type: 'item',
        title: 'Advisories',
        url: '/#content/repos',
        icon: IconExclamationCircle,
        badge: 5,
      },
      {
        type: 'item',
        title: 'Packages',
        url: '/#content/entitlements',
        icon: IconBox,
      },
      {
        type: 'item',
        title: 'Repositories',
        url: '/#content/repos',
        icon: IconDatabase,
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

export const AppSiteHeader = ({
  children,
  title,
}: PropsWithChildren<{ title: string; }>) => {
  return (
    <header className="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
      <div className="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
        <SidebarTrigger className="-ml-1" />

        <h1 className="text-base font-medium">{title}</h1>
        <div className="ml-auto flex items-center gap-2">
          {children}
        </div>
      </div>
    </header>
  );
};

const AppSidebarNavMain = () => {
  return sidebarItems.map((v, idx) => <AppSidebarNavItem key={idx} item={v} />);
};

const AppSidebarNavItem = ({ item }: { item: AppSidebarItem }) => {
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

      const badge = item.badge != undefined
        ? <Badge variant={item.badge == 0 ? "outline" : "default"} className="ml-auto">{item.badge}</Badge>
        : <></>

      return (
        <SidebarMenuItem>
          <SidebarMenuButton isActive={isActive} render={(
            <Link to={item.url}>
              {item.icon && <item.icon />}
              <span>{item.title}</span>
              {badge}
            </Link>
          )} />
        </SidebarMenuItem>
      );
  }
};

const AppSidebarNavUser = () => {
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
