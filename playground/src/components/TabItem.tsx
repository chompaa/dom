import { ReactNode } from "react";

const TabItem = ({
  children,
  // @ts-ignore
  label,
}: {
  children: ReactNode;
  label: string;
}) => (
  <div className="hide-scrollbar flex w-full flex-1 flex-col overflow-y-scroll p-1">
    {children}
  </div>
);

export default TabItem;
