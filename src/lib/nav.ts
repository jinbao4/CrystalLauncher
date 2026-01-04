export interface NavItem {
  label: string;
  location: string;
}

export const navItems: NavItem[] = [
  { label: "Play", location: "/" },
  { label: "Installations", location: "/install" },
  { label: "Settings", location: "/settings" },
];
