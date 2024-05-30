import { ReactNode } from "react";

const TabItem = ({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) => (
  <div className="flex flex-col flex-1 w-full p-1 overflow-y-scroll hide-scrollbar">
    {children}
  </div>
);

export default TabItem;
