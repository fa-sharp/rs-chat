import type * as React from "react";

import { Drawer, DrawerContent, DrawerTrigger } from "@/components/ui/drawer";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { useIsMobile } from "@/hooks/use-mobile";

interface Props {
  open?: boolean;
  onOpenChange?: React.Dispatch<React.SetStateAction<boolean>>;
  /** Trigger button */
  trigger: React.ReactNode;
  /** Props passed to the popover content */
  popoverProps?: React.ComponentProps<typeof PopoverContent>;
  children: React.ReactNode;
}

/** Responsive component that renders a popover on desktop and a drawer on mobile */
export default function PopoverDrawer({
  trigger,
  children,
  open,
  onOpenChange,
  popoverProps,
}: Props) {
  const isMobile = useIsMobile();

  if (!isMobile) {
    return (
      <Popover open={open} onOpenChange={onOpenChange}>
        <PopoverTrigger asChild>{trigger}</PopoverTrigger>
        <PopoverContent align="start" {...popoverProps}>
          {children}
        </PopoverContent>
      </Popover>
    );
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerTrigger asChild>{trigger}</DrawerTrigger>
      <DrawerContent>
        <div className="mt-4 border-t">{children}</div>
      </DrawerContent>
    </Drawer>
  );
}
